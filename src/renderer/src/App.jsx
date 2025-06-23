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
  const [syntax, setSyntax] = useState([]);

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
        const parserResult = await window.electron.runParser(editorContent);
        setTokens(result);
        setSyntax(parserResult);
        console.log(parserResult);
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
        <CodeEditor theme={theme} action={action} tokens={tokens} syntax={syntax} onContentChange={handleContentChange}></CodeEditor>
      </Box>
    </Box>
  )
}

export default App
