import { HStack, Button } from "@chakra-ui/react"
import "bootstrap-icons/font/bootstrap-icons.css";
import Dropdown from "./Dropdown";

function Nav(){
    return(
        <HStack wrap="wrap" justifyContent="space-between" gap="6" p={3} bg="#51557E">
            <Dropdown
                icon={<i className="bi bi-folder-fill"></i>}
            ></Dropdown>
            <HStack wrap="wrap" gap="3">
                <Button variant="ghost">
                    <i style={{fontSize: "1.5rem"}} className="bi bi-file-earmark-plus"></i>
                </Button>
                <Button variant="ghost">
                    <i style={{fontSize: "1.5rem"}} className="bi bi-upload"></i>
                </Button>
                <Button variant="ghost">
                    <i style={{fontSize: "1.5rem"}} className="bi bi-box-arrow-down"></i>
                </Button>
                <Button variant="ghost">
                    <i style={{fontSize: "1.5rem"}} className="bi bi-floppy"></i>
                </Button>
            </HStack>
            <Dropdown
                icon={<i className="bi bi-play-fill"></i>}
            ></Dropdown>
        </HStack>
    )
}

export default Nav
