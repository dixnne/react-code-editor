import { Box } from "@chakra-ui/react";
import "bootstrap-icons/font/bootstrap-icons.css";
import { useState } from "react";

function TabsMenu(){
    const [active, setActive] = useState(0)

    function handleClick(act) {
        setActive(act)
    }

    return(
        <Box bg="#816797">
            <ul className="nav justify-content-center py-2">
                <li className="nav-item">
                    <a className={"nav-link link-light " + (active==0? "bg-light rounded text-dark" : "")} role="button" onClick={() => handleClick(0)}>Lexical</a>
                </li>
                <li className="nav-item">
                    <a className={"nav-link link-light " + (active==1? "bg-light rounded text-dark" : "")} role="button" onClick={() => handleClick(1)}>Syntactic</a>
                </li>
                <li className="nav-item">
                    <a className={"nav-link link-light " + (active==2? "bg-light rounded text-dark" : "")} role="button" onClick={() => handleClick(2)}>Semantic</a>
                </li>
                <li className="nav-item">
                    <a className={"nav-link link-light " + (active==3? "bg-light rounded text-dark" : "")} role="button" onClick={() => handleClick(3)}>Intermediate Code</a>
                </li>
                <li className="nav-item">
                    <a className={"nav-link link-light " + (active==4? "bg-light rounded text-dark" : "")} role="button" onClick={() => handleClick(4)}>Execution</a>
                </li>
            </ul>
        </Box>
    )
}

export default TabsMenu
