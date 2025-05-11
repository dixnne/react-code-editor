import { HStack, Button } from "@chakra-ui/react";
import "bootstrap-icons/font/bootstrap-icons.css";
import Dropdown from "./Dropdown";
import OffCanvasMenu from "./OffCanvasMenu";
import { useState } from "react";
import Themes from "../assets/themes.js";



function Nav({ onAction, changeTheme }) {

    const [theme, setTheme] = useState("pointerOfDoom")

    function handleThemeChange(t) {
        setTheme(t)
        changeTheme(t)
    }

    function getThemes() {
        const themes = []
        for (const t in Themes) {
            const obj = {
                label: Themes[t].name, 
                action: () => handleThemeChange(Themes[t].code)
            }
            themes.push(obj)
        }
        console.log(Themes, themes);
        
        return themes
    }

    const handleAction = (action) => {
        onAction(null);
        setTimeout(() => onAction(action), 0);
    };

    const actions = [
        { icon: "bi-file-earmark-plus", label: "New File", action: () => handleAction("new-file") },
        { icon: "bi-box-arrow-down", label: "Open File", action: () => handleAction("open-file") },
        { icon: "bi-floppy", label: "Save", action: () => handleAction("save-file") },
        { icon: "bi-printer", label: "Save as", action: () => handleAction("save-file-as") },
        { icon: "bi-file-excel", label: "Close", action: () => handleAction("close-file") },
    ];

    return (
        <HStack wrap="wrap" justifyContent="space-between" gap="6" p={3} bg={Themes[theme].primary}>
            <HStack>
                <OffCanvasMenu></OffCanvasMenu>
                <Dropdown
                    icon={<i className="bi bi-folder-fill"></i>}
                    items={[
                        { label: "Open Folder", action: () => handleAction("open-folder") },
                        { label: "Save File", action: () => handleAction("save-file") },
                        { label: "Close Folder", action: () => handleAction("close-folder") }
                    ]}
                />
            </HStack>
            <HStack wrap="wrap" gap="3">
                {actions.map((item, index) => (
                    <Button key={index} variant="ghost" onClick={item.action} title={item.label}>
                        <i style={{ fontSize: "1.5rem" }} className={`bi ${item.icon}`}></i>
                    </Button>
                ))}
            </HStack>
            <HStack>
                <Dropdown
                    icon={<i className="bi bi-play-fill"></i>}
                    items={[
                        { label: "Run", action: () => handleAction("run") },
                        { label: "Debug", action: () => handleAction("debug") }
                    ]}
                />
                <Dropdown
                    icon={<i className="bi bi-palette"></i>}
                    items={getThemes()}
                />
            </HStack>
        </HStack>
    );
}

export default Nav;
