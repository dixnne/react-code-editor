// src/main.jsx

import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './assets/dreamc.css' // Asegúrate que esta es la ruta a tu CSS con Tailwind
import './main.css'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)