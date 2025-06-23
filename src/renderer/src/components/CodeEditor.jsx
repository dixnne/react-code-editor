import { Box, HStack, Stack } from "@chakra-ui/react";
import AceEditor from "react-ace-builds";
import React, { useState, useEffect, useRef } from "react";

import "react-ace-builds/node_modules/ace-builds/src-noconflict/theme-dracula";
import "react-ace-builds/node_modules/ace-builds/src-noconflict/ext-language_tools";
import DreamCMode from "../ace/mode/dreamc_mode";
import TabsMenu from "./TabsMenu";
import Themes from "../assets/themes.js";

function CodeEditor({ action, theme, tokens, syntax, onContentChange }) {
    const [editorContent, setEditorContent] = useState('');
    const [filePath, setFilePath] = useState(null);
    const [line, setLine] = useState(1);
    const [column, setColumn] = useState(1);
    const [active, setActive] = useState(0);

    const aceEditorRef = useRef(null);

    useEffect(() => {
        if (aceEditorRef.current) {
            const editor = aceEditorRef.current.editor;
            const session = editor.getSession();
            const customMode = new DreamCMode();
            session.setMode(customMode);
            console.log("Modo DreamC establecido manualmente.");
        } else {
            console.warn("Referencia a AceEditor no encontrada en useEffect.");
        }
    }, []);

    useEffect(() => {
         const electronAPI = window.electron;
         const runAction = async () => {
            if (action === "open-file") await openFile(electronAPI);
            if (action === "save-file") await saveFile(electronAPI);
            if (action === "save-file-as") await saveFileAs(electronAPI);
            if (action === "new-file") newFile();
            if (action === "close-file") closeFile();
         };
         runAction();
    }, [action]);

    const newFile = () => {
        setEditorContent("// Nuevo archivo DreamC\n");
        setFilePath(null);
        setLine(1); setColumn(1);
    };

    const closeFile = () => {
        setEditorContent("");
        setFilePath(null);
        setLine(1); setColumn(1);
    };

     const openFile = async (electronAPI) => {
        if (!electronAPI) return console.warn("Electron API not available.");
        try {
            const result = await electronAPI.openFile();
            if (result && typeof result.content === 'string' && typeof result.path === 'string') {
                setEditorContent(result.content);
                setFilePath(result.path);
                setLine(1); setColumn(1);
            } else if (result) {
                 console.warn("Resultado de openFile inválido:", result);
            }
        } catch (error) { console.error("Error opening file:", error); }
    };

    const saveFile = async (electronAPI) => {
        if (!electronAPI) return console.warn("Electron API not available.");
        if (!filePath) return saveFileAs(electronAPI);
        try {
            const success = await electronAPI.saveFile({ path: filePath, content: editorContent });
            if (success) console.log("File saved successfully!");
            else console.error("Failed to save file.");
        } catch (error) { console.error("Error saving file:", error); }
    };

    const saveFileAs = async (electronAPI) => {
        if (!electronAPI) return console.warn("Electron API not available.");
        try {
            const result = await electronAPI.saveFileAs({ content: editorContent });
            if (result?.path) {
                setFilePath(result.path);
                console.log("File saved successfully at:", result.path);
            } else if (result?.canceled) {
                console.log("Save As was canceled.");
            } else { console.error("Failed to save file as."); }
        } catch (error) { console.error("Error saving file as:", error); }
    };

    const handleChange = (value) => {
        setEditorContent(value);
        if (onContentChange) {
            onContentChange(value);
        }
    };

    const handleCursorChange = (selection) => {
        if (selection && typeof selection.getCursor === 'function') {
            const cursorPosition = selection.getCursor();
            if (cursorPosition) {
                 setLine(cursorPosition.row + 1);
                 setColumn(cursorPosition.column + 1);
            }
        }
    };

    const currentThemeUI = Themes && Themes[theme] ? Themes[theme] : { primary: '#343a40', secondary: '#6c757d', tertiary: '#e9ecef', secondarySemi: '#495057' };

    function printLexicalErrors(t) {
        if (!t || t.length === 0) return <div>No hay errores léxicos.</div>;
        return (
            t.map((token, index) => (
                token.tokenType === "Invalid" ? (
                    <p key={index}>Error: Invalid token "{token.lexeme}" at line {token.line} col {token.column}</p>
                ) : null
            ))
        );
    }

    // NEW FUNCTION: Print syntax errors
    function printSyntaxErrors(errors) {
        if (!errors || errors.length === 0) {
            return <div>No hay errores de sintaxis.</div>;
        }
        
        return errors.map((error, index) => (
            <p key={index}>
                Error: {error.error_type} - "{error.message}" at line {error.line} col {error.column}
            </p>
        ));
    }

    function renderActiveWindow(active) {
        switch (active) {
            case 0: // Lexical errors
                return (
                    <Box overflowY="auto" maxH="150px" px={3} bg={currentThemeUI.primary}>
                        {printLexicalErrors(tokens.tokens)}
                    </Box>
                );
            case 1: // Syntax errors (NEW TAB)
                return (
                    <Box overflowY="auto" maxH="150px" px={3} bg={currentThemeUI.primary}>
                        {printSyntaxErrors(syntax?.errors)}
                    </Box>
                );
            case 2: // Execution
                return <Box px={3} height="100%" bg={currentThemeUI.primary}>Execution</Box>;
            default:
                return <Box height="100%" bg={currentThemeUI.primary}></Box>;
        }
    }

    function handleClick(act) {
        setActive(act);
    }

    return (
        <Box height="100%">
            <HStack gap={0} alignItems="start" h={450}>
                <Stack gap={0} height="100%">
                    <AceEditor
                        ref={aceEditorRef}
                        width="55vw"
                        height="100%"
                        mode="text"
                        theme="dracula"
                        onChange={handleChange}
                        value={editorContent}
                        name="DREAMC_EDITOR"
                        editorProps={{ $blockScrolling: true }}
                        fontSize={14}
                        showPrintMargin={true}
                        showGutter={true}
                        highlightActiveLine={true}
                        setOptions={{
                            useWorker: false,
                            enableBasicAutocompletion: true,
                            enableLiveAutocompletion: true,
                            enableSnippets: true,
                            showLineNumbers: true,
                            tabSize: 4,
                        }}
                        onCursorChange={handleCursorChange}
                    />
                    <HStack height="40px" justifyContent="end" color="black" bg={currentThemeUI.tertiary} py={1} px={4}>
                        <span>Ln {line}, Col {column}</span>
                    </HStack>
                </Stack>
                <Box w="50vw" h="100%" bg={currentThemeUI.secondarySemi}>
                    <TabsMenu tokens={tokens} syntax={syntax} theme={theme} />
                </Box>
            </HStack>
            <Stack height="100%" bg={currentThemeUI.primary}>
                <Box color="white" bg={currentThemeUI.secondary} px={4} py={2}>
                    <ul className="nav">
                        <li className="nav-item">
                            <a className={"nav-link link-light " + (active === 0 ? "rounded bg-light text-dark" : "")} 
                               role="button" 
                               onClick={() => handleClick(0)}>
                                Lexical
                            </a>
                        </li>
                        {/* NEW SYNTAX ERRORS TAB */}
                        <li className="nav-item">
                            <a className={"nav-link link-light " + (active === 1 ? "rounded bg-light text-dark" : "")} 
                               role="button" 
                               onClick={() => handleClick(1)}>
                                Syntax
                            </a>
                        </li>
                        <li className="nav-item">
                            <a className={"nav-link link-light " + (active === 2 ? "rounded bg-light text-dark" : "")} 
                               role="button" 
                               onClick={() => handleClick(2)}>
                                Execution
                            </a>
                        </li>
                    </ul>
                </Box>
                <Box height={150}>
                    {renderActiveWindow(active)}
                </Box>
                <HStack justifyContent="end" bg={currentThemeUI.tertiary} color="black" px={4} py={2}>
                    <span>{filePath ? filePath.split(/[\\/]/).pop() : "Untitled"} - DreamC main</span>
                </HStack>
            </Stack>
        </Box>
    );
}

export default CodeEditor;