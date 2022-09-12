import { useEffect, useState } from 'react';

import { getGreetingFromContract, setGreetingOnContract, verifiedGreetingOnContract } from '../../utils/HelloNear';
import { EducationalText, SignInPrompt, SignOutButton } from './ui-components';
import { generateCalldata } from '../../circuit/generate-proof';

//css
import '../../assets/global.css';

const HelloNear = () => {
    const [valueFromBlockchain, setValueFromBlockchain] = useState();

    const [uiPleaseWait, setUiPleaseWait] = useState(true);

     // Get blockchian state once on component load
    useEffect(() => {
        getGreetingFromContract()
        .then(setValueFromBlockchain)
        .catch(alert)
        .finally(() => {
            setUiPleaseWait(false);
        });
    }, []);

    /// If user not signed-in with wallet - show prompt
    if (!window.walletConnection.isSignedIn()) {
        // Sign-in flow will reload the page later
        return <SignInPrompt greeting={valueFromBlockchain} />;
    }

    function changeGreeting(e) {
        e.preventDefault();
        setUiPleaseWait(true);
        const { greetingInput } = e.target.elements;
        setGreetingOnContract(greetingInput.value)
            .then(getGreetingFromContract)
            .then(setValueFromBlockchain)
            .catch(alert)
            .finally(() => {
            setUiPleaseWait(false);
            });
    }
    
    async function verifyGreeting(e) {
        e.preventDefault();
        setUiPleaseWait(true);
        const { greetingInput, a, b } = e.target.elements;

        const { proof, inputs } = await generateCalldata({ a: a.value, b: b.value })
        console.log("Proof:", proof);
        console.log("Inputs:", inputs);

        verifiedGreetingOnContract(greetingInput.value, proof, inputs)
            .then(getGreetingFromContract)
            .then(setValueFromBlockchain)
            .catch(alert)
            .finally(() => {
            setUiPleaseWait(false);
            });

        setUiPleaseWait(false);
    }
    
    return (
        <div>
            <SignOutButton accountId={window.accountId} />
            <main className={uiPleaseWait ? 'please-wait' : ''}>
                <h1>
                The contract says: <span className="greeting">{valueFromBlockchain}</span>
                </h1>
                <form onSubmit={verifyGreeting} className="change">
                <div>
                    <div>
                    <label>Change greeting:</label>
                    <input
                        autoComplete="off"
                        defaultValue={valueFromBlockchain}
                        id="greetingInput"
                    />
                    </div>
                    <div>
                    <label>Circuit inputs:</label><div>
                        <input
                        autoComplete="off"
                        defaultValue={3}
                        id="a"
                        />
                        <input
                        autoComplete="off"
                        defaultValue={11}
                        id="b"
                        />
                    </div>
                    </div>
                </div>
                <div>
                    <button>
                    <span>Save</span>
                    <div className="loader"></div>
                    </button>
                </div>
                </form>
                <EducationalText />
            </main>
        </div>
    )
}

export default HelloNear;