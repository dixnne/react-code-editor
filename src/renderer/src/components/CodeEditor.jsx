import { Box, HStack, Stack } from "@chakra-ui/react";
import AceEditor from "react-ace-builds";
import "react-ace-builds/webpack-resolver-min.js";
import { useState, useEffect } from "react";
import TabsMenu from "./TabsMenu";
import Themes from "../assets/themes.js";

function CodeEditor({ action, theme }) {
    const [editorContent, setEditorContent] = useState("Start coding!");
    const [filePath, setFilePath] = useState(null);
    const [line, setLine] = useState(1);
    const [column, setColumn] = useState(1);
    const [active, setActive] = useState(0)

    function handleClick(act) {
        setActive(act)
    }

    useEffect(() => {
        if (action === "open-file") openFile();
        if (action === "save-file") saveFile();
        if (action === "save-file-as") saveFileAs();
        if (action === "new-file") newFile();
        if (action === "close-file") closeFile();
    }, [action]);

    const newFile = async () => {
        setEditorContent("");
        setFilePath(null);
    };

    const closeFile = async () => {
        setEditorContent("");
        setFilePath(null);
    };

    const openFile = async () => {
        if (window.electron) {
            const result = await window.electron.openFile();
            if (result) {
                setEditorContent(result.content);
                setFilePath(result.path);
            }
        }
    };

    const saveFile = async () => {
        if (!filePath) {
            await saveFileAs();
            return;
        }
        if (window.electron) {
            const success = await window.electron.saveFile({ path: filePath, content: editorContent });
            if (success) {
                console.log("File saved successfully!");
            }
        }
    };

    const saveFileAs = async () => {
        if (window.electron) {
            const result = await window.electron.saveFileAs({ content: editorContent });
            if (result) {
                setFilePath(result.path);
                console.log("File saved successfully at:", result.path);
            }
        }
    };

    const handleChange = (value) => {
        setEditorContent(value);
    };

    const handleCursorChange = (selection) => {
        const newLine = selection.cursor.row + 1;
        const newColumn = selection.cursor.column;
        setLine(newLine);
        setColumn(newColumn);
    };

    return (
        <Box height="100%">
            <HStack gap={0} alignItems="start" h={450}>
                <Stack gap={0} height="100%">
                    <AceEditor
                        width="55vw"
                        height="100%"
                        mode="c-mode"
                        theme="github"
                        onChange={handleChange}
                        name="UNIQUE_ID_OF_DIV"
                        editorProps={{ $blockScrolling: true }}
                        fontSize={14}
                        showPrintMargin={true}
                        showGutter={true}
                        highlightActiveLine={true}
                        value={editorContent}
                        setOptions={{
                            hScrollBarAlwaysVisible: true,
                            vScrollBarAlwaysVisible: true,
                            enableBasicAutocompletion: false,
                            enableLiveAutocompletion: false,
                            enableSnippets: false,
                            showLineNumbers: true,
                            tabSize: 2,
                        }}
                        onCursorChange={handleCursorChange}
                    />
                    <HStack height="40px" justifyContent="end" color="black" bg={Themes[theme].tertiary} py={1} px={4}>
                        <span>Ln {line}, col {column}</span>
                    </HStack>
                </Stack>
                <Box w="50vw" h="100%" bg={Themes[theme].secondarySemi}>
                    <TabsMenu theme={theme} />
                </Box>
            </HStack>
            <Stack height="100%" bg={Themes[theme].primary}>
                <Box color="white" bg={Themes[theme].secondary} px={4} py={2}>
                    <ul className="nav">
                        <li className="nav-item">
                            <a className={"nav-link link-light " + (active==0? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(0)}>Errors</a>
                        </li>
                        <li className="nav-item">
                            <a className={"nav-link link-light " + (active==1? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(1)}>Execution</a>
                        </li>
                    </ul>
                </Box>
                <Box height={150}></Box>
                <HStack justifyContent="end" bg={Themes[theme].tertiary} color="black" px={4} py={2}>
                    <span>C main</span>
                </HStack>
            </Stack>
        </Box>
    );
}

export default CodeEditor;