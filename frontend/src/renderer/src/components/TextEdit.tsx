import React, { useState, useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

import "./LoadingScreen"

import "../styles/TextEdit.css";
import LoadingScreen from "./LoadingScreen";


export default function TextEdit(): React.JSX.Element {
  const [loaded, setLoaded] = useState<boolean>(false);
  const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const edit_ref = useRef<HTMLDivElement | null>(null);
  const pending_inserts = useRef(0);

  useEffect(() => {
    if (canvas_ref.current === null || edit_ref.current === null) {
      return;
    }

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

    const getCaretPosition = (el: HTMLElement): number => {
      const selection = window.getSelection();
      if (
        !selection ||
        selection.rangeCount === 0 ||
        !el.contains(selection.anchorNode)
      ) {
        return 0;
      }

      const range = selection.getRangeAt(0);
      const textNode = ensureStructure(el);
      if (range.startContainer === textNode) {
        return range.startOffset;
      }
      if (range.startContainer === el) {
        return range.startOffset === 0 ? 0 : textNode.length;
      }
      return textNode.length;
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

    const replaceContentWithState = (newText: string): void => {
      const el = edit_ref.current!;
      const selection = window.getSelection();
      let savedPos = 0;
      let hasFocus = false;
      if (
        selection &&
        selection.rangeCount > 0 &&
        el.contains(selection.anchorNode)
      ) {
        savedPos = getCaretPosition(el);
        hasFocus = true;
      }
      el.textContent = newText;
      if (hasFocus) {
        const safePos = Math.min(savedPos, newText.length);
        const newTextNode = ensureStructure(el);
        setCaret(newTextNode, safePos);
      }
    };

    const handlerRemove = (position: number): void => {
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

    const handleInsert = (position: number, char: string): void => {
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

    const handleFullSync = (new_text: string): void => {
      const currentText = edit_ref.current!.innerText;
      if (new_text === currentText) {
        return;
      }
      replaceContentWithState(new_text);
    };

    window.api.onRemoveRequest(handlerRemove);
    window.api.onInsertRequest(handleInsert);
    window.api.onFullSync(handleFullSync);


    const isSupportedChar = (char: string): boolean => {
      if (char.length !== 1) return false;
      const code = char.codePointAt(0);
      return code !== undefined && code >= 32 && code <= 126;
    };

    const handleKeyDown = (event: KeyboardEvent): void => {
      if ((event.ctrlKey || event.metaKey) && ["c", "a"].includes(event.key)) {
        console.error("Unhandled user input");
        return;
      }

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
        if (
          !selection ||
          selection.rangeCount === 0 ||
          !selection.isCollapsed
        ) {
           if (!selection?.isCollapsed) {
            event.preventDefault();
            console.error("Selection range is not supported");
          }
          return;
        }

        const el = edit_ref.current!;
        const cursor_pos = getCaretPosition(el);

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

    setTimeout(() => {
      setLoaded(true);
    }, 1000);

    return () => {
      el.removeEventListener("keydown", handleKeyDown);
      el.removeEventListener("mouseup", handleMouse);
      sandbox.destroy?.();
    };
  }, []);

  return (
    <>
      {
        !loaded ? ( <LoadingScreen/> ) : ( null )
      }
      <canvas ref={canvas_ref} className="glslCanvas"/>
      <div
        ref={edit_ref}
        className="text-field"
        contentEditable="plaintext-only"
        spellCheck={false}
      />
    </>
  );
}
