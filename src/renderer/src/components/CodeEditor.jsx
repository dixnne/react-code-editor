import { Box, HStack, Stack } from "@chakra-ui/react";
import AceEditor from "react-ace-builds";
import "react-ace-builds/webpack-resolver-min";
import { useState, useEffect } from "react";
import TabsMenu from "./TabsMenu";

function CodeEditor({ action }) {
    const [editorContent, setEditorContent] = useState("Start coding!");
    const [filePath, setFilePath] = useState(null);
    const [line, setLine] = useState(1);
    const [column, setColumn] = useState(1); 

    useEffect(() => {
        if (action === "open-file") openFile();
        if (action === "save-file") saveFile();
        if (action === "new-file") newFile();
    }, [action]);

    const newFile = async () => {
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
        if (window.electron) {
            if (filePath) {
                const success = await window.electron.saveFile({ path: filePath, content: editorContent });
                if (success) {
                    console.log("File saved successfully!");
                }
            } else {
                const result = await window.electron.saveFileAs({ content: editorContent });
                if (result) {
                    setFilePath(result.path);
                    console.log("File saved successfully at:", result.path);
                }
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
            <HStack alignItems="start" h={450}>
                <Stack height="100%">
                    <AceEditor
                        width="50vw"
                        height="100%"
                        mode="c"
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
                            enableBasicAutocompletion: false,
                            enableLiveAutocompletion: false,
                            enableSnippets: false,
                            showLineNumbers: true,
                            tabSize: 2,
                        }}
                        onCursorChange={handleCursorChange}
                    />
                    <HStack height="40px" justifyContent="end" color="black" bg="#D6D5A8" py={1} px={4}>
                        <span>Ln {line}, col {column}</span>
                    </HStack>
                </Stack>
                <Box w="50vw" h="100%" bg="#81679740">
                    <TabsMenu />
                </Box>
            </HStack>
            <Stack height="100%" mt={3} bg="#51557E">
                <Box color="white" bg="#816797" px={4} py={2}>
                    <span>Errors</span>
                </Box>
                <Box height={150}></Box>
                <HStack justifyContent="end" bg="#D6D5A8" color="black" px={4} py={2}>
                    <span>C main</span>
                </HStack>
            </Stack>
        </Box>
    );
}

export default CodeEditor;
