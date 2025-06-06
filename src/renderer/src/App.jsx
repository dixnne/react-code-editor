import { Box } from "@chakra-ui/react";
import CodeEditor from "./components/CodeEditor";
import Nav from "./components/Nav";
import { useEffect, useState } from "react";
import Themes from "./assets/themes.js";


function App() {
  const [action, setAction] = useState(null)
  const [theme, setTheme] = useState("pointerOfDoom")
  const [editorContent, setEditorContent] = useState("");
  // eslint-disable-next-line no-unused-vars
  const [tokens, setTokens] = useState([]);

  function handleThemeChange(t) {
    setTheme(t);
  }

  function handleContentChange(content) {
    setEditorContent(content);
  }

  
  useEffect(() => {
    if (editorContent.trim() === "") return;
    const fetchTokens = async () => {
      try {
        const result = await window.electron.runLexer(editorContent);
        setTokens(result);
        console.log("Tokens:", result);
      } catch (error) {
        console.error("Error fetching tokens:", error);
      }
    };
    fetchTokens();
  }, [editorContent]);
 

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
        <CodeEditor theme={theme} action={action} tokens={tokens} onContentChange={handleContentChange}></CodeEditor>
      </Box>
    </Box>
  )
}

export default App
