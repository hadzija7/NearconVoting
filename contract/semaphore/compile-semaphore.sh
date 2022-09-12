#!/bin/bash

#export NODE_OPTIONS="--max-old-space-size=16384"

cd circuits
mkdir -p build

if [ -f ./pot14_final.ptau ]; then
    echo "pot14_final.ptau already exists. Skipping."
else
    echo 'Downloading semaphores pot14_final.ptau'
    wget https://storage.googleapis.com/trustedsetup-a86f4.appspot.com/ptau/pot14_final.ptau
fi

echo "Compiling: circuit..."

# compile circuit
circom semaphore.circom --r1cs --wasm --sym -o build
snarkjs r1cs info build/semaphore.r1cs

# Start a new zkey and make a contribution
snarkjs groth16 setup build/semaphore.r1cs pot14_final.ptau build/semaphore_0000.zkey
snarkjs zkey contribute build/semaphore_0000.zkey build/semaphore_final.zkey --name="1st Contributor Name" -v -e="random text"
snarkjs zkey export verificationkey build/semaphore_final.zkey build/verification_key.json

# # Generate proof
# node build/semaphore_js/generate_witness.js build/semaphore_js/semaphore.wasm input.json witness.wtns
# snarkjs groth16 prove build/circuit_final.zkey witness.wtns proof.json public.json
