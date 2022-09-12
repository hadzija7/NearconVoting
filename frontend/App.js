import 'regenerator-runtime/runtime';
import React from 'react';
//react router
import { BrowserRouter as Router, Switch, Route, Link } from 'react-router-dom';
//import components
import HelloNear from './components/HelloNear';
// import Polls from './components/Polls';
import Poll from './components/Polls/Poll';
// import About from './components/About';

export default function App() {
  return (
    <Router>
        <div>
            <h2>Voting app</h2>
            <nav className="navbar navbar-expand-lg navbar-light bg-light">
            <ul className="navbar-nav mr-auto">
                <li><Link to={'/'} className="nav-link"> Home </Link></li>
                {/* <li><Link to={'/about'} className="nav-link">About</Link></li> */}
                <li><Link to={'/polls'} className="nav-link">Polls</Link></li>
            </ul>
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
