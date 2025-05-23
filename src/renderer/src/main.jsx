import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { ColorModeScript, ChakraProvider } from "@chakra-ui/react";
import theme from './theme';
import './assets/dreamc.css'
import 'bootstrap/dist/css/bootstrap.css'
import "bootstrap/dist/js/bootstrap.bundle.min";

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <ColorModeScript initialColorMode={theme.config.initialColorMode} />
    <ChakraProvider theme={theme}>
      <App />
    </ChakraProvider>
  </React.StrictMode>
)
