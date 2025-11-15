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

    doc.insertAbsoluteWrapper(0, "a");
    doc.insertAbsoluteWrapper(1, "b");
    doc.insertAbsoluteWrapper(2, "c");
    console.log(doc.collectString());
    doc.removeAbsolute(3);
    console.log(doc.collectString());

    const sandbox: GlslCanvas = new GlslCanvas(canvas_ref.current);
    sandbox.load(backdrop_shader);

    edit_ref.current.addEventListener("input", (event: Event) => {
      const input_type: string = event.inputType;
      if (input_type == "insertText") {
        console.log("Insert");
      } else if (
        input_type == "deleteContentBackward" ||
        input_type == "deleteContentForward"
      ) {
        console.log("Delete");
      }
      // console.log(document.getSelection()?.getRangeAt(0).startOffset);
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
