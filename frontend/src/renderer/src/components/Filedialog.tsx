import { useEffect, useRef } from "react";

import "../styles/FieDialog.css";

export default function FileDialog({
    active,
    onExit
}: {
    active: boolean,
    onExit: () => void
}): React.JSX.Element {
    let container_ref = useRef<HTMLDivElement | null>(null);
    let modal_ref = useRef<HTMLDivElement | null>(null);
    let email_input_ref = useRef<HTMLDivElement | null>(null);

    console.log(active);

    const handleClick = (event: MouseEvent) => {
        if (modal_ref.current && !modal_ref.current.contains(event.target)) {
            onExit();
        }
    };

    useEffect(() => {
        window.addEventListener("mousedown", handleClick);
        return () => { window.removeEventListener("mousedown", handleClick); }
    }, []);
    
    return (
        <div className={ active ? "modal-container active" : "modal-container" } ref={ container_ref }>
            <div className="order-modal" ref={ modal_ref }>
                <div className="textfield">
                    <h2>Email:</h2>
                    <div contentEditable="true" className="text-input" ref={ email_input_ref }></div>                    
                </div>
                <button onClick={ () => alert("Saved") }>Save</button>
            </div>
        </div>
    )
}