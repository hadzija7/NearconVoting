import 'regenerator-runtime/runtime';
//react imports
import React from 'react';
import { BrowserRouter as Router, Switch, Route, Link } from 'react-router-dom';
import { useState } from 'react';
//import components
import HelloNear from './components/HelloNear';
import Poll from './components/Polls/Poll';
//semaphore imports
import { Identity } from "@semaphore-protocol/identity"
//util imports
import { registerIdentity as registerID } from './utils/Semaphore';

export default function App() {
  const [registered, setRegistered] = useState(false)

  const registerIdentity = async () => {
    console.log("Registering...")
    //generate identity commitment
    const identity = new Identity()
    const commitment = identity.generateCommitment()
    //call the utils method
    await registerID(commitment.toString())
    //store the identity in the local storage
    localStorage.setItem('idCommitment', JSON.stringify(commitment))
    //set the state variable
    setRegistered(true)
  }

  const renderRegisterButton = () => {
    let result
    if(registered){
      result = <label>Registered</label>
    }else{
      result = <button onClick={registerIdentity}>Register</button>
    }
    return result
  }

  return (
    <Router>
        <div>
            <h2>Voting app</h2>
            <nav className="navbar navbar-expand-lg navbar-light bg-light">
              <div className="navbar-nav mr-auto">
                <div><Link to={'/'} className="nav-link"> Home </Link></div>
                <div><Link to={'/polls'} className="nav-link">Polls</Link></div>
                <div>{renderRegisterButton()}</div>
              </div>
            </nav>
            <hr />
            <Switch>
                <Route exact path='/' component={HelloNear} />
                <Route path='/polls' component={Poll} />
                {/* <Route path='/about' component={About} /> */}
            </Switch>
      </div>
    </Router>
  );
}
