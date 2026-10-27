"""Independent R04 closed-form anchors; no Rust or sampler imports.

Geant4 11.4.1: source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc
GetIndependentReactionTime and SamplePDC; dna/utils/src/G4DNAMolecularReactionTable.cc
ComputeEffectiveRadius and SetReactionType. Literal inputs: data/chemistry/*.toml
(provenance there names G4EmDNAChemistry_option3 and dissociation/molecule sources).

TDC F=min(1,W erfc(|L-a|/(2 sqrt(Dt)))), W=a/L.
PDC F=W[erfc(x)-exp(-x²)erfcx(x+A sqrt(Dt))], x=b/sqrt(Dt).
Neutral: L=r, W=(a/r)kobs/kdiff, A=kact/(kobs*a), b=(r-a)/2.
Charged: use the source's sinh/coth/alpha equations directly, independently
of Rust's reduced expressions. L=rc/expm1(rc/r), R=rc/expm1(rc/a).
These charged laws are Geant4 effective-distance laws, NOT physical Coulomb
first-passage solutions. No physical-Coulomb PASS is inferred here.
"""
import argparse
import json
from pathlib import Path
import tomllib
import mpmath as mp

ROOT = Path(__file__).resolve().parents[2]
IDS = [1, 2, 8, 9, 11, 17, 42, 51]
TAU = ['0.01', '0.1', '1', '10', '100']


def generate(dps):
    mp.mp.dps = dps
    m = mp.mpf
    na = m('6.02214076e23')
    rc_unit = m('1.602176634e-19') * m('299792458')**2 * m('1e-7') / m('8.617333e-5') / (m('293.15')*m('80.1'))*m('1e9')
    def read(name):
        return tomllib.loads((ROOT / 'data/chemistry' / name).read_text(encoding='utf-8'), parse_float=m)
    species = {s['tag']: s for s in read('species_option3.toml')['species']}
    reactions = {r['id']: r for r in read('reactions_option3.toml')['reaction']}
    cases = []
    for id in IDS:
        r = reactions[id]
        p, q = [species[s] for s in r['reactants']]
        cases.append(dict(name=f'option3-{id}', id=id, aa=p==q, pdc=r['set_reaction_type'],
                          d=(p['diffusion_m2_per_s']+q['diffusion_m2_per_s'])*m('1e6'),
                          rc=p['charge']*q['charge']*rc_unit,
                          radius=p['van_der_waals_radius_nm']+q['van_der_waals_radius_nm'],
                          k=r['rate_per_molar_s']*m('1e12')/na))
    for name, template, aa, pdc, sign in [('neutral-tdc-ab',2,False,False,0),('charged-tdc-aa',8,True,False,1),('neutral-pdc-aa',17,True,True,0),('charged-pdc-ab',42,False,True,-1)]:
        # Physical embedding with length unit rc_unit for charged cases, so rc/a=±1.
        a = rc_unit if sign else m(1)
        rc = sign*rc_unit
        R = rc/mp.expm1(rc/a) if rc else a
        k = 4*mp.pi*(m('.5') if aa else 1)*(R/2 if pdc else a)
        cases.append(dict(name='syn-'+name,id=template,aa=aa,pdc=pdc,d=m(1),rc=rc,radius=a,k=k))
    output=[]
    for c in cases:
        d,rc,k = c['d'],c['rc'],c['k']
        factor=4*mp.pi*d*(m('.5') if c['aa'] else 1)
        a=c['radius'] if c['pdc'] else k/factor
        R=rc/mp.expm1(rc/a) if rc and c['pdc'] else a
        kd=factor*R
        points=[]
        for ratio in map(m,['1','1.5','3']):
            r=ratio*a
            L=rc/mp.expm1(rc/r) if rc else r
            w=R/L
            if c['pdc']:
                ka=k*kd/(kd-k)
                w*=k/kd
                if rc:
                    v=ka/(4*mp.pi*a*a*mp.exp(-rc/a))
                    alpha=v+rc*d/(a*a*(1-mp.exp(-rc/a)))
                    A=4*a*a*alpha/(d*rc*rc)*mp.sinh(rc/(2*a))**2
                    b=rc/4*(mp.coth(rc/(2*r))-mp.coth(rc/(2*a)))
                else:
                    A=ka/(k*a)
                    b=(r-a)/2
            probs=[]
            for tau in map(m,TAU):
                root=a*mp.sqrt(tau)
                if c['pdc']:
                    x=b/root
                    z=x+A*root
                    f=w*(mp.erfc(x)-mp.exp(z*z-x*x)*mp.erfc(z))
                else:
                    f=min(m(1),w*mp.erfc(abs(L-a)/(2*root)))
                probs.append(float(f))
            points.append(dict(ratio=float(ratio),probabilities=probs+[float(1-min(m(1),w))]))
        output.append(dict(name=c['name'],id=c['id'],radius=float(a),diffusion=float(d),
                           rate_per_molar_s=float(k*na/m('1e12')),points=points))
    return output


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    anchors=generate(80)
    # Round only after evaluation; independently repeat at doubled precision.
    assert anchors == generate(160), 'anchors unstable under precision doubling'
    threshold=float(mp.sqrt(2)*mp.erfinv(1-mp.mpf('.001')/216))
    result=dict(mpmath_version=mp.__version__,precision_digits=[80,160],geant4_version='11.4.1',
                law='Geant4 effective-distance (charged); diffusion radiation boundary (neutral)',
                family_alpha=.001,endpoints=216,z_threshold=threshold,cases=anchors)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')

if __name__ == '__main__':
    main()
