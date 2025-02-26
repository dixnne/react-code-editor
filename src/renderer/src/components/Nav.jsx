import { HStack, Button } from "@chakra-ui/react";
import "bootstrap-icons/font/bootstrap-icons.css";
import Dropdown from "./Dropdown";

function Nav({ onAction }) {
    const handleAction = (action) => {
        onAction(null);
        setTimeout(() => onAction(action), 0);
    };

    const actions = [
        { icon: "bi-file-earmark-plus", label: "New File", action: () => handleAction("new-file") },
        { icon: "bi-upload", label: "Open File", action: () => handleAction("open-file") },
        { icon: "bi-box-arrow-down", label: "Download", action: () => handleAction("download") },
        { icon: "bi-floppy", label: "Save", action: () => handleAction("save-file") },
    ];

    return (
        <HStack wrap="wrap" justifyContent="space-between" gap="6" p={3} bg="#51557E">
            <Dropdown
                icon={<i className="bi bi-folder-fill"></i>}
                items={[
                    { label: "Open Folder", action: () => handleAction("open-folder") },
                    { label: "Save File", action: () => handleAction("save-file") },
                    { label: "Close Folder", action: () => handleAction("close-folder") }
                ]}
            />
            <HStack wrap="wrap" gap="3">
                {actions.map((item, index) => (
                    <Button key={index} variant="ghost" onClick={item.action} title={item.label}>
                        <i style={{ fontSize: "1.5rem" }} className={`bi ${item.icon}`}></i>
                    </Button>
                ))}
            </HStack>
            <Dropdown
                icon={<i className="bi bi-play-fill"></i>}
                items={[
                    { label: "Run", action: () => handleAction("run") },
                    { label: "Debug", action: () => handleAction("debug") }
                ]}
            />
        </HStack>
    );
}

export default Nav;
