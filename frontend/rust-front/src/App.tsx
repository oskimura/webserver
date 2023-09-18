import { useState, useEffect } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import axios from 'axios';
import './App.css'

function App() {
  const url = 'http://localhost/api/todo'
  const [count, setCount] = useState(0)

  useEffect(() => {
    axios.get(url).then((res) => {
      console.log(res.data)
    }).catch((err) => {console.log(err);});
  },[])

  return (
    <div className="App">
      helloworld
    </div>
  )
}

export default App
