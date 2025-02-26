import { Box } from "@chakra-ui/react";
import CodeEditor from "./components/CodeEditor";
import Nav from "./components/Nav";
import { useState } from "react";


function App() {
  const [action, setAction] = useState(null);
  return (
    <Box
      minH="100vh"
      bg="#1F2544"
    >
      <Nav onAction={setAction}></Nav>
      <Box
        color="#FFD0EC"
        px={6}
        py={8}
        height="100%"
      >
        <CodeEditor action={action}></CodeEditor>
      </Box>
    </Box>
  )
}

export default App
