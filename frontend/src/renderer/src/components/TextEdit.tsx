import React, { useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

import "../styles/TextEdit.css";

export default function TextEdit(): React.JSX.Element {
  const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const edit_ref = useRef<HTMLDivElement | null>(null);
  const pending_inserts = useRef(0);

  useEffect(() => {
    if (canvas_ref.current === null 
      || edit_ref.current === null) { return; }

    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);

    const ensureStructure = (el: HTMLElement): Text => {
      let textNode = el.firstChild;
      if (!textNode || textNode.nodeType !== Node.TEXT_NODE) {
        textNode = document.createTextNode("");
        el.prepend(textNode);
      }

      if (!el.lastElementChild || el.lastElementChild.tagName !== "BR") {
        const br = document.createElement("br");
        el.appendChild(br);
      }
      return textNode as Text;
    };

    const setCaret = (textNode: Text, pos: number): void => {
      const selection = window.getSelection();
      const range = document.createRange();
      const safePos = Math.min(pos, textNode.length);
      try {
        range.setStart(textNode, safePos);
        range.setEnd(textNode, safePos);
        selection?.removeAllRanges();
        selection?.addRange(range);
      } catch (e) {
        console.error(e);
      }
    };

    const removeHandler = (position: number): void => {
      pending_inserts.current = 0;
      if (position <= 0) return;
      const el = edit_ref.current!;
      const textNode = ensureStructure(el);
      try {
        if (position - 1 < textNode.length) {
          textNode.deleteData(position - 1, 1);
        }
        setCaret(textNode, position - 1);
      } catch (e) {
        console.warn(e);
      }
    };

    const insertHandler = (position: number, char: string): void => {

      const el = edit_ref.current!;
      const textNode = ensureStructure(el);

      try {
        pending_inserts.current = Math.max(0, pending_inserts.current - 1);
        const safePos = Math.min(position, textNode.length);
        textNode.insertData(safePos, char);
        setCaret(textNode, safePos + 1);
        if (el.scrollHeight > el.clientHeight) {
          el.scrollTop = el.scrollHeight;
        }
      } catch (e) {
        console.warn(e);
      }
    };

    window.api.onRemoveRequest(removeHandler);
    window.api.onInsertRequest(insertHandler);

    const isSupportedChar = (char: string): boolean => {
      if (char.length !== 1) return false;
      const code = char.codePointAt(0);
      return code !== undefined && code >= 32 && code <= 126;
    };

    const handleKeyDown = (event: KeyboardEvent): void => {
      if ((event.ctrlKey || event.metaKey)
        && ["c", "a"].includes(event.key)
      ) { console.error("Unhandled user input"); return; }

      if (event.key.startsWith("Arrow")) {
        pending_inserts.current = 0;
        return;
      }

      const isBackspace = event.key === "Backspace";
      const isEnter = event.key === "Enter";
      const isChar = event.key.length === 1;

      if (isBackspace || isEnter || isChar) {
        if (isChar && !isEnter && !isSupportedChar(event.key)) {
          event.preventDefault();
          console.log("Unsupported user input");
          return;
        }

        const selection = document.getSelection();
        if (!selection || selection.rangeCount === 0) { return; }

        if (!selection.isCollapsed) {
          event.preventDefault();
          console.error("Selection is not supported");          
          return;
        }

        const range = selection.getRangeAt(0);
        const el = edit_ref.current!;
        const textNode = ensureStructure(el);

        let cursor_pos = 0;
        if (range.startContainer === textNode) {
          cursor_pos = range.startOffset;
        } else {
          cursor_pos = textNode.length;
        }

        let effective_pos = cursor_pos;
        if (isEnter || isChar) {
          effective_pos += pending_inserts.current;
          pending_inserts.current++;
        }

        event.preventDefault();

        window.api.onUserKeydown(event.key, effective_pos);
      }
    };

    const handleMouse = (): void => {
      pending_inserts.current = 0;
    };

    const el = edit_ref.current;
    el.addEventListener("keydown", handleKeyDown);
    el.addEventListener("mouseup", handleMouse);
    ensureStructure(el);

    return () => {
      el.removeEventListener("keydown", handleKeyDown);
      el.removeEventListener("mouseup", handleMouse);
      sandbox.destroy?.();
    };
  }, []);

  return (
    <>
      <canvas ref={canvas_ref} className="glslCanvas" />
      <div
        ref={edit_ref}
        className="text-field"
        contentEditable="plaintext-only"
        spellCheck={false}
      />
    </>
  );
}