import React, { useState, useEffect, useRef } from "react";
import GlslCanvas from "glslCanvas";
import { backdrop_shader } from "@renderer/assets/backdrop_shader";

export default function TextEdit(): React.JSX.Element {
  const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const edit_ref = useRef<HTMLDivElement | null>(null);
  const [doc, setDoc] = useState<ReturnType<
    typeof window.api.createDocument
  > | null>(null);

  useEffect(() => {
    setDoc(window.api.createDocument());
  }, []);

  useEffect(() => {
    if (doc === null) {
      return;
    }
    if (canvas_ref.current === null || edit_ref.current === null) {
      return;
    }

    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);
    edit_ref.current.addEventListener("input", (event: Event) => {
      const cursor_pos = document.getSelection()?.getRangeAt(0).startOffset;
      const input_type: string = event.inputType;
      console.log(cursor_pos + " " + event.data);
      if (input_type == "insertText") {
        doc.insertAbsoluteWrapper(cursor_pos - 1, event.data);
      } else if (input_type == "deleteContentBackward") {
        doc.removeAbsoluteWrapper(cursor_pos as number);
      }
      console.log(doc.collectString());
    });
  }, [doc]);

  return (
    <>
      <canvas ref={canvas_ref} className="glslCanvas" />
      <div
        ref={edit_ref}
        className="text-field"
        contentEditable="plaintext-only"
      />
      ;
    </>
  );
}
