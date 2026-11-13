"""Validate the prospective scheduler CSVs; incomplete inputs are inconclusive."""
import csv
import hashlib
import json
import math
import sys
from pathlib import Path
from rr008_seed_audit import seed_pair

HEADER = 'seed1,seed2,channel,orientation,trial,outcome,a_nm,d1_nm2_ps,d2_nm2_ps,p1x,p1y,p1z,p2x,p2y,p2z,event_time_ps,event_step_ps,r1x,r1y,r1z,r2x,r2y,r2z,products,px,py,pz,parents_removed'.split(',')
N = 2048
ALPHA = 0.001 / 6 / 2


class Incomplete(ValueError):
    pass


def cdf(k, p):
    if k < 0:
        return 0.0
    if k >= N:
        return 1.0
    logs = [math.lgamma(N+1)-math.lgamma(i+1)-math.lgamma(N-i+1)+i*math.log(p)+(N-i)*math.log1p(-p) for i in range(k+1)]
    m = max(logs)
    return math.exp(m) * math.fsum(math.exp(x-m) for x in logs)


def frequency(count):
    # CP upper < p iff P_p[X <= count] < alpha/2; lower > p iff P_p[X >= count] < alpha/2.
    lower_gt = lambda p: cdf(count-1, p) > 1-ALPHA
    upper_lt = lambda p: cdf(count, p) < ALPHA
    if lower_gt(.1) and upper_lt(.25):
        return 'PASS'
    if upper_lt(.1) or lower_gt(.25):
        return 'FAIL'
    return 'INCONCLUSIVE'


def close(x, y):
    return math.isfinite(x) and abs(x-y) <= 1e-12 + 1e-12*abs(y)


def validate(path, replica, trials=2048):
    data = Path(path).read_bytes()
    if len(data) > 10485760:
        raise ValueError('oversize')
    lines = data.decode('ascii').splitlines()
    if len(lines) != 3 * trials + 4 or lines[0] != '# schema: syn_scheduler_followup_v1; Geant4: 1141' or lines[1] != f'# replica: {replica}; histories: {trials}; birth_ps: 1; horizon_ps: 1000000' or lines[-1] != f'# complete: true; replica: {replica}; histories: {trials}; rows: {3 * trials}':
        raise Incomplete('incomplete schema or footer')
    rows = list(csv.reader(lines[2:-1]))
    if rows[0] != HEADER or len(rows) != 3 * trials + 1:
        raise Incomplete('invalid header or row count')
    counts = [0, 0, 0]
    violation = False
    for index, row in enumerate(rows[1:]):
        o, h = divmod(index, trials)
        if len(row) != 28 or row[:5] != [*map(str, seed_pair(replica, o, h)), '11', str(o), str(h)]:
            raise ValueError('seed, order, or schema mismatch')
        origin = (4.1, -2.87, 1.23) if o == 2 else (0., 0., 0.)
        second = tuple(origin[i] + (1.23 if i == (2 if o == 1 else 0) else 0.) for i in range(3))
        expected = (.41, .0022, .007, *origin, *second)
        try:
            if not all(close(float(x), y) for x, y in zip(row[6:15], expected)):
                raise ValueError('fixed input mismatch')
            if row[5] == 'censored':
                if row[15:] != ['NA']*13:
                    raise ValueError('invalid censored row')
            elif row[5] == 'reaction':
                counts[o] += 1
                t, step = map(float, row[15:17])
                positions = list(map(float, row[17:23]))
                if not (math.isfinite(t) and math.isfinite(step) and 1 < t <= 1000000 and step > 0 and all(close(x,y) for x,y in zip(positions, (*origin,*second))) and row[23:] == ['0','NA','NA','NA','1']):
                    violation = True
            else:
                raise ValueError('unknown outcome')
        except (OverflowError, ValueError) as exc:
            raise ValueError(f'invalid row {index}: {exc}') from exc
    return counts, violation


def evidence(path):
    """Check declared run metadata and hashes; declarations still require human audit."""
    with open(path, encoding='utf-8') as f:
        record = json.load(f)
    required = ('geant4_version', 'g4emlow_version', 'physics', 'chemistry',
                'timestep', 'date', 'platform', 'compiler', 'generator_commit',
                'generator_dirty', 'source_path', 'source_sha256', 'binary_path',
                'binary_sha256', 'runs')
    if any(k not in record for k in required):
        raise Incomplete('missing provenance fields')
    for key, value in (('geant4_version', '11.4.1'), ('g4emlow_version', '8.8'),
                       ('physics', 'G4EmDNAPhysics_option2'),
                       ('chemistry', 'G4EmDNAChemistry_option3'), ('timestep', 'IRT_syn')):
        if record[key] != value:
            raise ValueError(f'wrong provenance {key}')
    for key in ('date', 'platform', 'compiler', 'generator_commit'):
        if not isinstance(record[key], str) or not record[key].strip():
            raise Incomplete(f'missing provenance {key}')
    if not isinstance(record['generator_dirty'], bool):
        raise Incomplete('generator_dirty must be boolean')
    for prefix in ('source', 'binary'):
        digest = hashlib.sha256(Path(record[prefix + '_path']).read_bytes()).hexdigest()
        if digest != record[prefix + '_sha256']:
            raise ValueError(f'{prefix} hash mismatch')
    runs = record['runs']
    if not isinstance(runs, list) or len(runs) != 4:
        raise Incomplete('four runs required')
    results = []
    for entry, (replica, trials) in zip(runs, ((1, 1), (2, 1), (1, 2048), (2, 2048))):
        try:
            output, manifest = entry['output'], entry['manifest']
            if entry['replica'] != replica or entry['histories'] != trials:
                raise ValueError('run order or settings mismatch')
            if entry['exit_code'] != 0 or not (0 <= entry['seconds'] <= 7200) or not (0 < entry['peak_bytes'] <= 8 * 1024**3):
                raise Incomplete('run failed or exceeded limits')
            expected_command = [record['binary_path'], 'followup', output, manifest, str(replica), str(trials)]
            if entry['command'] != expected_command:
                raise ValueError('command mismatch')
            if hashlib.sha256(Path(output).read_bytes()).hexdigest() != entry['output_sha256']:
                raise ValueError('output hash mismatch')
            manifest_bytes = Path(manifest).read_bytes()
            if hashlib.sha256(manifest_bytes).hexdigest() != entry['manifest_sha256']:
                raise ValueError('manifest hash mismatch')
            expected = ''.join(f'{a} {b}\n' for a, b in
                               (seed_pair(replica, o, h) for o in range(3) for h in range(N))).encode('ascii')
            if manifest_bytes != expected:
                raise ValueError('manifest does not match frozen derivation')
            result = validate(output, replica, trials)
            if trials == 1 and result[1]:
                raise Incomplete('smoke structural check failed')
            if trials == N:
                results.append(result)
        except (KeyError, TypeError) as exc:
            raise Incomplete(f'missing or invalid run evidence: {exc}') from exc
    return results


def main():
    if len(sys.argv) != 2:
        raise ValueError('usage: rr008_followup_consumer.py EVIDENCE.json')
    try:
        results = evidence(sys.argv[1])
    except (OSError, UnicodeError, Incomplete, json.JSONDecodeError) as exc:
        print(f'INCONCLUSIVE: {exc}')
        return
    except ValueError as exc:
        print(f'FAIL: invalid complete data: {exc}')
        return
    counts = results[0][0] + results[1][0]
    rates = [frequency(c) if c else 'INCONCLUSIVE' for c in counts]
    verdict = 'FAIL' if any(v for _,v in results) or 'FAIL' in rates else 'PASS' if all(counts) and all(x == 'PASS' for x in rates) else 'INCONCLUSIVE'
    print(f'{verdict}: counts={counts}, frequency={rates}')


if __name__ == '__main__':
    main()
