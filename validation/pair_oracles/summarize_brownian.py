"""Summarize retained R04 diagnostics without converting censored evidence to PASS."""
import argparse
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    provenance = json.loads((args.directory / "provenance.json").read_text())
    if not provenance["completed"] or not provenance["unchanged_sources"]:
        raise ValueError("incomplete run or changed source: no complete-family summary")
    records = [json.loads(line) for line in (args.directory / "estimates.jsonl").read_text().splitlines()]
    summaries = [r for r in records if r["record"] == "summary"]
    if len(summaries) != 36 or len({(r['case'], r['distance']) for r in summaries}) != 36:
        raise ValueError("missing or duplicated case/distance summaries")
    rows = []
    contact_counterexamples = []
    for summary in summaries:
        selected = [r for r in records if r.get("case") == summary["case"]
                    and r.get("distance") == summary["distance"]]
        finite = [r for r in selected if r["record"] == "finite"
                  and r["n"] == summary["n"] and r["level"] == summary["level"]]
        eventual = [r for r in selected if r["record"] == "eventual"
                    and r["n"] == summary["n"] and r["level"] == summary["level"]]
        if len(finite) != 5 or len(eventual) != 1:
            raise ValueError("missing final endpoints")
        if summary['brownian_verdict'] == 'PASS(exact-physical-contact-only)':
            assert eventual[0]['nonreaction_mc_interval'] == [0.0, 0.0]
            for endpoint in finite:
                assert endpoint['count'] == endpoint['n']
                difference = 1.0 - endpoint['source_law_anchor']
                if abs(difference) >= .005:
                    contact_counterexamples.append(dict(case=summary['case'], distance=1.0,
                        tau=endpoint['tau'], physical_cdf=1.0,
                        source_law_cdf=endpoint['source_law_anchor'], absolute_difference=abs(difference)))
            if abs(eventual[0]['source_law_anchor']) >= .005:
                contact_counterexamples.append(dict(case=summary['case'], distance=1.0,
                    endpoint='eventual nonreaction', physical_nonreaction=0.0,
                    source_law_nonreaction=eventual[0]['source_law_anchor'],
                    absolute_difference=abs(eventual[0]['source_law_anchor'])))
        sizes = sorted({r['n'] for r in selected if r['record'] == 'finite'})
        if sizes != [n for n in [10000,20000,40000,80000,160000,320000,640000,1000000]
                     if n <= summary['n']]:
            raise ValueError("nonregistered precision sequence")
        for n in sizes:
            levels = sorted({r['level'] for r in selected if r['record'] == 'finite' and r['n'] == n})
            if levels != list(range(max(levels)+1)) or len(levels) < 3:
                raise ValueError("missing registered minimum levels")
        rows.append(dict(case=summary['case'], distance=summary['distance'], n=summary['n'],
                         final_level=summary['level'], sizes=sizes,
                         finite_refined=sum(r['refined'] for r in finite),
                         finite_diagnostics=summary['finite_physical_diagnostics'],
                         maximum_absolute_difference=max(abs(r['difference']) for r in finite),
                         nonreaction_identification=eventual[0]['nonreaction_identification'],
                         nonreaction_mc_interval=eventual[0]['nonreaction_mc_interval'],
                         physical_nonreaction_anchor=eventual[0]['physical_anchor'],
                         unresolved=eventual[0]['unresolved'], return_bound=eventual[0]['return_bound'],
                         scientific_verdict=summary['brownian_verdict']))
    result = dict(complete_available_schedule_execution=True,
                  numerical_oracle_completion="INCONCLUSIVE",
                  physical_agreement_with_source_law="FAIL" if contact_counterexamples else "INCONCLUSIVE",
                  exact_contact_counterexamples=contact_counterexamples, g1="NOT_PASS", rows=rows,
                  finite_rows=sum(r['record']=='finite' for r in records),
                  eventual_rows=sum(r['record']=='eventual' for r in records),
                  notes=["The eventual endpoint has population censoring intervals, not observed Bernoulli nonreaction outcomes.",
                         "Fixed-z finite diagnostics do not have a simultaneous sequential family guarantee.",
                         "PDE errors are empirical; no continuum Brownian discretization enclosure has been proven.",
                         "Completing this computational schedule does not complete the preregistered eventual comparison.",
                         "Exact-contact falsification compares physical trajectories with the source-law anchors, not a new direct production-sampler test."],
                  estimates_sha256=hashlib.sha256((args.directory/'estimates.jsonl').read_bytes()).hexdigest())
    with args.output.open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('rows','exact_contact_counterexamples')}, indent=2))
    for r in rows:
        print(r['case'], r['distance'], r['n'], r['final_level'], r['finite_refined'],
              r['finite_diagnostics'], r['nonreaction_identification'])


if __name__ == '__main__':
    main()
