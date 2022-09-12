import { Worker, NEAR, NearAccount } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';

import { readFileSync } from 'fs';

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async (t) => {
  // Init the worker and start a Sandbox server
  const worker = await Worker.init();

  // Deploy contract
  const root = worker.rootAccount;
  const contract = await root.createSubAccount('test-account');
  // Get wasm file path from package.json test script in folder above
  await contract.deploy(
    process.argv[2],
  );

  // Save state for test runs, it is unique for each test
  t.context.worker = worker;
  t.context.accounts = { root, contract };
});

test.afterEach(async (t) => {
  // Stop Sandbox server
  await t.context.worker.tearDown().catch((error) => {
    console.log('Failed to stop the Sandbox:', error);
  });
});

test('easy', async (t) => {
  const { contract } = t.context.accounts;

  const res = await contract.view('get_next_leaf');

  t.is(res, 0);
});

test('verify proof', async (t) => {
  const { contract } = t.context.accounts;

  const proof_str: string = readFileSync('../contract/circuits/proof.json', 'utf-8');
  const pub_input_str: string = readFileSync('../contract/circuits/public.json', 'utf-8');
  const res: boolean = await contract.view('verify_proof_on_chain', { proof: proof_str, inputs: pub_input_str });

  t.is(res, true);
});

test('invalid proof', async (t) => {
  const { contract } = t.context.accounts;

  const proof_str: string = readFileSync('../contract/circuits/proof.json', 'utf-8');
  const pub_input_str: string = '["0","0","0","0"]';
  const res: boolean = await contract.view('verify_proof_on_chain', { proof: proof_str, inputs: pub_input_str });

  t.is(res, false);
});