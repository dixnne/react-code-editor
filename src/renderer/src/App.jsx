import { Box } from "@chakra-ui/react";
import CodeEditor from "./components/CodeEditor";
import Nav from "./components/Nav";
import { useState } from "react";
import Themes from "./assets/themes.js";


function App() {
  const [action, setAction] = useState(null)
  const [theme, setTheme] = useState("pointerOfDoom")

  function handleThemeChange(t) {
    setTheme(t);
  }

  return (
    <Box
      minH="100vh"
      bg={Themes[theme].background}
    >
      <Nav changeTheme={handleThemeChange} onAction={setAction}></Nav>
      <Box
        color={Themes[theme].text}
        px={6}
        py={8}
        height="100%"
      >
        <CodeEditor theme={theme} action={action}></CodeEditor>
      </Box>
    </Box>
  )
}

export default App
