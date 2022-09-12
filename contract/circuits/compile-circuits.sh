#!/bin/bash

#export NODE_OPTIONS="--max-old-space-size=16384"

cd circuits
mkdir -p build

if [ -f ./powersOfTau28_hez_final_14.ptau ]; then
    echo "powersOfTau28_hez_final_14.ptau already exists. Skipping."
else
    echo 'Downloading powersOfTau28_hez_final_14.ptau'
    wget https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_14.ptau
fi

echo "Compiling: circuit..."

# compile circuit
circom circuit.circom --r1cs --wasm --sym -o build
snarkjs r1cs info build/circuit.r1cs

# Start a new zkey and make a contribution
snarkjs groth16 setup build/circuit.r1cs powersOfTau28_hez_final_14.ptau build/circuit_0000.zkey
snarkjs zkey contribute build/circuit_0000.zkey build/circuit_final.zkey --name="1st Contributor Name" -v -e="random text"
snarkjs zkey export verificationkey build/circuit_final.zkey build/verification_key.json

# # Generate proof
# node build/circuit_js/generate_witness.js build/circuit_js/circuit.wasm input.json witness.wtns
# snarkjs groth16 prove build/circuit_final.zkey witness.wtns proof.json public.json
