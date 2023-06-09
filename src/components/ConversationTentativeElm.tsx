import { queryRinna } from "@/hooks/rinna-hook";

interface ConversationTentativeElmProps {
  id: number;
  prompt: string;
}

export default function ConversationTentativeElm({
  id,
  prompt,
}: ConversationTentativeElmProps) {
  const rinnaResponse = queryRinna(id, prompt).replace(/<NL>/g, "\n");

  const text = rinnaResponse.includes("</s>")
    ? rinnaResponse.replace(/<\/s>/g, "")
    : `${rinnaResponse}▊`;
  return (
    <>
      ========== <br />
      User: {prompt}
      <br />
      Rinna: {text}
      <br />
    </>
  );
}
