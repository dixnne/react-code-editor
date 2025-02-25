/* eslint-disable prettier/prettier */
import { Box, HStack, Stack } from "@chakra-ui/react";
import AceEditor from "react-ace-builds";
import "react-ace-builds/webpack-resolver-min";
import { useState } from "react"
import TabsMenu from "./TabsMenu";

function CodeEditor() {

    const [editorSelectionObject, setEditorSelectionObject] = useState({});

    function onChange(newValue) {
        console.log('change', newValue);
        const cursor = editorSelectionObject.getCursor();
        console.log(cursor);
    }

    function onCursorChange(selection) {
        console.log(selection.getCursorPosition());
        
    }
    return (
        <Box height="100%">
            <HStack alignItems="start" h={450}>
                <Stack height="100%">
                    <AceEditor
                        width="50vw"
                        height="100%"
                        mode="c"
                        theme="github"
                        onSelection={onCursorChange}
                        onChange={onChange}
                        onSelectionChange={(selectionObj) => {
                            setEditorSelectionObject(selectionObj);
                        }}
                        name="UNIQUE_ID_OF_DIV"
                        editorProps={{ $blockScrolling: true }}
                        fontSize={14}
                        showPrintMargin={true}
                        showGutter={true}
                        highlightActiveLine={true}
                        value="Start coding!"
                        setOptions={{
                            enableBasicAutocompletion: false,
                            enableLiveAutocompletion: false,
                            enableSnippets: false,
                            showLineNumbers: true,
                            tabSize: 2,
                    }}
                    />
                    <HStack height="40px" justifyContent="end" color="black" bg="#D6D5A8" py={1} px={4}>
                        <span>Ln ----, col -----</span>
                    </HStack>
                </Stack>
                <Box w="50vw" h="100%" bg="#81679740">
                    <TabsMenu></TabsMenu>
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
    )
}

export default CodeEditor
