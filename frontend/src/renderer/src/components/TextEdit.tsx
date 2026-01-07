import React, { useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

export default function TextEdit(): React.JSX.Element {
  const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const edit_ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (canvas_ref.current === null 
      || edit_ref.current === null) { return; }

    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);

    const setCaretPosition = (element: HTMLElement, position: number) => {
      const range = document.createRange();
      const selection = window.getSelection();
      const textNode = element.firstChild;
      if (textNode) {
        const safePos = Math.min(position, textNode.textContent?.length || 0);
        range.setStart(textNode, safePos);
        range.setEnd(textNode, safePos);
        selection?.removeAllRanges();
        selection?.addRange(range);
      }
    };

    window.api.onRemoveRequest((position: number) => {
      if (position <= 0) return;
      const el = edit_ref.current!;
      const content = el.textContent || "";
      el.textContent = content.slice(0, position - 1) + content.slice(position);
      setCaretPosition(el, position - 1);
    });
    
    window.api.onInsertRequest((position: number, char: string) => {
      const el = edit_ref.current!;
      const content = el.textContent || "";
      el.textContent =
        content.slice(0, position) + char + content.slice(position) + "\n";
      setCaretPosition(el, position + 1);
    });
    
    const handleKeyDown = (event: KeyboardEvent): void => {
      if (!event.key.startsWith("Arrow")) { event.preventDefault(); }
      if (event.key.length === 1
          || event.key === "Backspace"
          || event.key === "Enter"
      ) {
        const selection = document.getSelection();
        if (!selection || selection.rangeCount === 0) return;
        const cursor_pos = selection.getRangeAt(0).startOffset;
        window.api.onUserKeydown(event.key, cursor_pos);
      }
    };

    edit_ref.current.addEventListener("keydown", handleKeyDown);

    return () => {
      edit_ref.current?.removeEventListener("keydown", handleKeyDown);
      sandbox.destroy?.();
    };

  }, []);

  return (
    <>
      <canvas ref={canvas_ref} className="glslCanvas"/>
      <div ref={edit_ref} className="text-field" contentEditable="plaintext-only"/>
    </>
  );
}