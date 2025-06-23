import { Box, Stack } from "@chakra-ui/react";
import "bootstrap-icons/font/bootstrap-icons.css";
import { useState } from "react";
import Themes from "../assets/themes.js";
import TokenTable from "./TokenTable.jsx";
import TreeView from './TreeView.jsx';

function TabsMenu({tokens, syntax, theme}){
    const [active, setActive] = useState(0)

    function handleClick(act) {
        setActive(act)
    }

    function setActiveWindow(act) {
        switch(act){
            case 0:
                return <Box height="100%" bg={Themes[theme].background}>
                    <TokenTable tokens={tokens} />
                </Box>
            case 1:
                return <Box height="100%" bg={Themes[theme].background}>
                        <TreeView data ={syntax}></TreeView>
                    </Box>
            case 2:
                return <Box height="100%" bg={Themes[theme].background}>Semantic Analysis</Box>
            case 3:
                return <Box height="100%" bg={Themes[theme].background}>Intermediate Code</Box>
            case 4:
                return <Box height="100%" bg={Themes[theme].background}>Hash Table</Box>
            default:
                return <Box height="100%" bg={Themes[theme].background}></Box>
        }
    }

    return(
        <Stack height="100%" bg={Themes[theme].primary}>
            <Box bg={Themes[theme].secondary}>
                <ul className="nav justify-content-center py-2">
                    <li className="nav-item">
                        <a className={"nav-link link-light " + (active==0? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(0)}>Lexical</a>
                    </li>
                    <li className="nav-item">
                        <a className={"nav-link link-light " + (active==1? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(1)}>Syntactic</a>
                    </li>
                    <li className="nav-item">
                        <a className={"nav-link link-light " + (active==2? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(2)}>Semantic</a>
                    </li>
                    <li className="nav-item">
                        <a className={"nav-link link-light " + (active==3? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(3)}>Intermediate Code</a>
                    </li>
                    <li className="nav-item">
                        <a className={"nav-link link-light " + (active==4? "rounded bg-light text-dark" : "")} role="button" onClick={() => handleClick(4)}>Hash Table</a>
                    </li>
                </ul>
            </Box>
            { setActiveWindow(active) }
        </Stack>      
    )
}

export default TabsMenu
