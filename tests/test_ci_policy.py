"""CI structure plus real Git and actual Bash admission; no product mocks."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = {'workflow': 'checks.yml', 'gate': 'stable', 'gate_name': 'Stable UI and Rust verification', 'workers': ['javascript', 'distribution', 'rust'], 'results': {'JS_RESULT': 'javascript', 'PYTHON_RESULT': 'distribution', 'RUST_RESULT': 'rust'}, 'budgets': {'scope': 5, 'javascript': 10, 'distribution': 10, 'rust': 20, 'stable': 5}, 'caches': 1, 'once': ['node --test tests/*.test.mjs', "unittest discover -s tests -p 'test_release_*.py'", 'cargo test --locked -p cfvigil']}
WRITER = "${{ github.ref == 'refs/heads/dev' && github.event_name != 'pull_request' }}"
CACHE = 'Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0'

def workflow(name):
    return yaml.load((ROOT / '.github/workflows' / name).read_text(), Loader=yaml.BaseLoader)

class CIPolicyTests(unittest.TestCase):
    def setUp(self):
        self.doc = workflow(SPEC['workflow'])
        self.gate = self.doc['jobs'][SPEC['gate']]

    def test_GivenOrdinaryJobs_WhenParsed_ThenBudgetsDAGAndTrustRemainStrict(self):
        self.assertEqual(self.doc['permissions'], {'contents': 'read'})
        self.assertEqual(self.doc['on']['push'], {'branches':['dev'], 'paths':['ci/run-preflight.txt']})
        self.assertEqual(set(self.doc['jobs']), set(SPEC['budgets']))
        self.assertEqual(self.doc['concurrency']['cancel-in-progress'], 'true')
        self.assertNotIn('paths', self.doc['on']['pull_request'])
        for name, budget in SPEC['budgets'].items():
            job = self.doc['jobs'][name]
            self.assertEqual(job['timeout-minutes'], str(budget), name)
            self.assertNotIn('environment', job)
            self.assertNotIn('continue-on-error', job)
            for step in job.get('steps', []):
                self.assertNotIn('continue-on-error', step)
                if step.get('uses', '').startswith('actions/checkout@'):
                    self.assertEqual(step['with']['persist-credentials'], 'false')
            if name in SPEC['workers']:
                self.assertEqual(job['needs'], 'scope')
                self.assertEqual(job['if'], "needs.scope.outputs.required == 'true'")
        self.assertEqual(self.gate['name'], SPEC['gate_name'])
        self.assertEqual(self.gate['if'], 'always()')
        self.assertEqual(self.gate['needs'], ['scope', *SPEC['workers']])
        expected = {'SCOPE_RESULT':'${{ needs.scope.result }}', 'REQUIRED':'${{ needs.scope.outputs.required }}'}
        expected.update({key:'${{ needs.' + job + '.result }}' for key,job in SPEC['results'].items()})
        self.assertEqual(self.gate['env'], expected)
        self.assertNotIn('${{ secrets.', str(self.doc))

    def test_GivenCachesAndCommands_WhenParsed_ThenOnlyTrustedWritersAndOneStablePassRemain(self):
        caches = [s['with'] for j in self.doc['jobs'].values() for s in j.get('steps', []) if s.get('uses') == CACHE]
        self.assertEqual(len(caches), SPEC['caches'])
        self.assertEqual(len({c['shared-key'] for c in caches}), len(caches))
        for cache in caches:
            self.assertEqual(cache['save-if'], WRITER)
        commands = '\n'.join(s.get('run','') for j in self.doc['jobs'].values() for s in j.get('steps', []))
        self.assertNotIn('seq 1 10', commands)
        self.assertNotIn('continue-on-error', commands)
        for command in SPEC['once']:
            self.assertEqual(commands.count(command), 1, command)
        for line in commands.splitlines():
            if any('cargo ' + verb in line for verb in ('test ', 'check ', 'clippy ')):
                self.assertIn('--locked', line)

    def test_GivenDevelopmentCandidates_WhenParsed_ThenOnlyTheirOwnMarkerCanBuildPrivatePackages(self):
        for name,marker in (('package-candidate.yml','ci/run-package-candidate.txt'),('windows-candidate.yml','ci/run-windows-candidate.txt')):
            with self.subTest(workflow=name):
                doc=workflow(name)
                self.assertEqual(doc['on'],{'push':{'branches':['dev'],'paths':[marker]}})
                self.assertEqual(doc['permissions'],{'contents':'read'})
                for job in doc['jobs'].values():self.assertNotIn('environment',job)
                self.assertNotIn('CFVIGIL_R2_',str(doc))
                self.assertNotIn('CFVIGIL_APT_SIGNING',str(doc))

    def run_gate(self, values):
        self.assertEqual(len(self.gate['steps']), 1)
        return subprocess.run(['bash','-c',self.gate['steps'][0]['run']], env={**os.environ,**values},
                              capture_output=True, text=True, timeout=10).returncode

    def test_GivenDependencyOutcomes_WhenAdmissionRuns_ThenOnlyExactApplicableEvidencePasses(self):
        keys = list(SPEC['results'])
        for required, expected in (('true','success'), ('false','skipped')):
            values = dict(SCOPE_RESULT='success', REQUIRED=required, **dict.fromkeys(keys,expected))
            self.assertEqual(self.run_gate(values), 0)
            for key in keys:
                for result in ('success','failure','cancelled','skipped',''):
                    if result == expected: continue
                    with self.subTest(required=required,key=key,result=result):
                        self.assertNotEqual(self.run_gate({**values,key:result}),0)
        for result,required in (('failure','true'),('cancelled','true'),('skipped','false'),('','true'),('success',''),('success','unknown')):
            self.assertNotEqual(self.run_gate(dict(SCOPE_RESULT=result, REQUIRED=required, **dict.fromkeys(keys,'success'))),0)

    def exercise(self, paths, operation='add', event='pull_request'):
        with tempfile.TemporaryDirectory(prefix='ci-scope-') as temp:
            root=Path(temp)
            def git(*args):
                return subprocess.check_output(['git','-c','commit.gpgsign=false','-c','core.hooksPath=/dev/null',
                    '-c','user.name=Fixture','-c','user.email=fixture@invalid.example',*args],cwd=root,stderr=subprocess.DEVNULL,text=True)
            git('init','-q','-b','base')
            (root/'initial').write_text('base\n')
            if operation in ('rename','delete'):
                p=root/paths[0];p.parent.mkdir(parents=True,exist_ok=True);p.write_text('fixture\n'*20)
            git('add','-A');git('commit','-qm','base')
            if event=='pull_request':
                git('checkout','-qb','changes')
                if operation=='add':
                    for name in paths:
                        p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('changed\n')
                elif operation=='rename':
                    (root/paths[1]).parent.mkdir(parents=True,exist_ok=True);git('mv',*paths)
                elif operation=='delete': (root/paths[0]).unlink()
                git('add','-A');git('commit','--allow-empty','-qm','change')
                git('checkout','-q','base');git('merge','--no-ff','-qm','merge','changes')
            output=root/'output'
            run=subprocess.run(['bash',str(ROOT/'scripts/ci-pr-scope.sh')],cwd=root,
                env={**os.environ,'GITHUB_EVENT_NAME':'pull_request' if event=='unmerged' else event,'GITHUB_OUTPUT':str(output)},
                capture_output=True,text=True)
            return run.returncode,output.read_text() if output.exists() else ''

    def test_GivenRealGitChanges_WhenClassified_ThenOnlyKnownProseSkipsAndBothRenameSidesCount(self):
        cases=[(['README.md','docs/guide.md'],'add',False),(['docs/config.json'],'add',True),
          (['docs/prototypes/bus.conf'],'add',True),(['apps/desktop/ui/README.md'],'add',True),
          (['.github/workflows/recover-release.yml'],'add',True),(['scripts/helper.py'],'add',True),(['.unknown'],'add',True),(['other.md'],'add',True),
          (['src/line\nname.rs'],'add',True),(['docs/a.md','src/a.rs'],'add',True),
          (['src/a.rs','docs/a.md'],'rename',True),(['docs/a.md','src/a.rs'],'rename',True),
          (['docs/old.md','docs/new.md'],'rename',False),(['src/a.rs'],'delete',True),(['docs/a.md'],'delete',False)]
        for paths,operation,required in cases:
            with self.subTest(paths=paths,operation=operation):
                code,output=self.exercise(paths,operation)
                self.assertEqual(code,0)
                self.assertEqual(output,'required='+str(required).lower()+'\n')

    def test_GivenMissingMetadataOrExplicitDispatch_WhenClassified_ThenNoFalseSkipIsProduced(self):
        for event in ('push','workflow_dispatch'):
            self.assertEqual(self.exercise([],event=event),(0,'required=true\n'))
        for event in ('schedule','unmerged','pull_request',''):
            self.assertNotEqual(self.exercise([],event=event)[0],0)
        env={**os.environ,'GITHUB_EVENT_NAME':'workflow_dispatch'};env.pop('GITHUB_OUTPUT',None)
        run=subprocess.run(['bash',str(ROOT/'scripts/ci-pr-scope.sh')],env=env,capture_output=True,text=True)
        self.assertNotEqual(run.returncode,0)

if __name__=='__main__': unittest.main()
