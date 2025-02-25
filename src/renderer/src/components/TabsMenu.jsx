import { Box } from "@chakra-ui/react";
import "bootstrap-icons/font/bootstrap-icons.css";

function TabsMenu(){
    return(
        <Box bg="#816797">
            <ul className="nav justify-content-center">
                <li className="nav-item">
                    <a className="nav-link link-light">Lexical</a>
                </li>
                <li className="nav-item">
                    <a className="nav-link link-light">Syntactic</a>
                </li>
                <li className="nav-item">
                    <a className="nav-link link-light">Semantic</a>
                </li>
                <li className="nav-item">
                    <a className="nav-link link-light">Intermediate Code</a>
                </li>
                <li className="nav-item">
                    <a className="nav-link link-light">Execution</a>
                </li>
            </ul>
        </Box>
    )
}

export default TabsMenu
