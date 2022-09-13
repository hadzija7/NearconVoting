import { Identity } from "@semaphore-protocol/identity"
import { Group } from "@semaphore-protocol/group"
import { generateProof, verifyProof } from "@semaphore-protocol/proof"

import { useEffect, useState } from "react";

import verificationKey from "../../circuit/verification_key.json";

const Poll = () => {

    // const [proposals, setProposals] = (['option1', 'option2', 'option3'])
    let proposals = ['option1', 'option2', 'option3']

    const votingProcess = async (event) => {
        event.preventDefault()
        //generate identity and identity commitment
        const identity = new Identity()
        const trapdoor = identity.getTrapdoor()
        const nullifier = identity.getNullifier()
        const commitment = identity.generateCommitment()
        console.log("Commitment: ", commitment)

        //create a new semaphore group. at the end this should be an on-chain merkle tree
        const group = new Group()
        //add member to the group for this poll
        group.addMember(commitment)
        console.log("Group: ", group)

        //external nullifier should be pollId (something that uniquely represents poll)
        const externalNullifier = BigInt(1)
        const signal = "Hello world"

        const fullProof = await generateProof(identity, group, externalNullifier, signal, {
            zkeyFilePath: "./circuit_final.zkey",
            wasmFilePath: "./circuit.wasm"
        })
        console.log("Full proof: ", fullProof)

        await verifyProof(verificationKey, fullProof)

        // console.log(result)
    }

    useEffect(() => {
    }, []);

    return (
        <div style={{display:"flex", flexDirection:"column", justifyContent:"center", margin:"auto 0"}}>
            <div>
                <h1>Poll example</h1>
            </div>
            <div style={{alignSelf:"center"}}>
                Vote for one of the 3 options
            </div>
            <form onSubmit={votingProcess} style={{alignSelf:"center"}}>
                <div>
                    {proposals.map( (proposal) => (
                        <div key={proposal}>
                            <input value={proposal} type="radio" name="vote" />
                            proposal
                        </div>
                    ))}
                </div>
                <button style={{marginTop:"20px"}}>Vote</button>
            </form>
        </div>
    )
}

export default Poll;