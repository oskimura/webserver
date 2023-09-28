import { useState, useEffect } from 'react'
//import reactLogo from './assets/react.svg'
//import viteLogo from '/vite.svg'
import axios from 'axios';
import './App.css'

function App() {
  //const [count, setCount] = useState(0)
  useEffect(() => {
    axios.get('/api/todo').then((res) => {
      console.log(res.data)
    }).catch((err) => {console.log(err);});
  },[])

  const [sqlQuery, setSqlQuery] = useState('');
  const [result, setResult] = useState<string | null>(null);

  const handleButtonClick = async () => {
    try {
      const response = await axios.post('/api/parse', { sql: sqlQuery });
      setResult(response.data);
    } catch (error) {
      console.error('API error:', error);
      setResult('API request failed');
    }
  };

  return (
    <div>
      <h1>SQL Parser</h1>
      <div>
        <textarea
          rows={5}
          cols={40}
          value={sqlQuery}
          onChange={(e) => setSqlQuery(e.target.value)}
          placeholder="Enter SQL query..."
        />
      </div>
      <div>
        <button onClick={handleButtonClick}>Parse</button>
      </div>
      <div>
        <h2>Result:</h2>
        {result !== null ? <pre>{result}</pre> : <p>No result yet</p>}
      </div>
    </div>
  );
}

export default App
