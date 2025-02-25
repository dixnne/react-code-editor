import { Box } from "@chakra-ui/react";
import CodeEditor from "./components/CodeEditor";
import Nav from "./components/Nav";


function App() {
  return (
    <Box
      minH="100vh"
      bg="#1F2544"
    >
      <Nav></Nav>
      <Box
        color="#FFD0EC"
        px={6}
        py={8}
        height="100%"
      >
        <CodeEditor></CodeEditor>
      </Box>
    </Box>
  )
}

export default App
