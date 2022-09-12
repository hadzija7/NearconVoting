import { Identity } from "@semaphore-protocol/identity"
import { Group } from "@semaphore-protocol/group"
import { generateProof, verifyProof } from "@semaphore-protocol/proof"

import { useEffect } from "react";

const Poll = () => {
    const votingProcess = async () => {
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
            zkeyFilePath: "../../circuits/semaphore/semaphore_final.zkey",
            wasmFilePath: "../../circuits/semaphore/semaphore.wasm"
        })
        console.log("Full proof: ", fullProof)

        const verificationKey = JSON.parse(fs.readFileSync("../../circuits/semaphore/verification_key.json", "utf-8"))

        await verifyProof(verificationKey, fullProof)

        // console.log(result)
    }

    useEffect(() => {
        votingProcess()
    }, []);

    return (
        <div>
            This is a poll
        </div>
    )
}

export default Poll;