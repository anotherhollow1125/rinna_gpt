import { useState, useEffect } from "react";
import PromptFieldElm from "@/components/PromptFieldElm";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useMemo } from "react";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import ConversationTentativeElm from "@/components/ConversationTentativeElm";
import useRinnaStandby from "@/hooks/rinna-standby-hook";

interface Query {
  id: number;
  prompt: string;
}

function App() {
  // https://amateur-engineer.com/react-mui-dark-mode/
  const prefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)");
  const theme = useMemo(
    () =>
      createTheme({
        palette: {
          mode: prefersDarkMode ? "dark" : "light",
        },
      }),
    [prefersDarkMode]
  );

  const [conversationList, setConversationList] = useState<Query[]>([]);
  const [rinnaStandby, setRinnaStandby] = useRinnaStandby();
  const [queryId, setQueryId] = useState(0);

  const send = (prompt: string) => {
    setConversationList([...conversationList, { id: queryId, prompt }]);
    setQueryId(queryId + 1);
  };

  return (
    <ThemeProvider theme={theme}>
      <div>
        <PromptFieldElm
          send={send}
          standby={rinnaStandby}
          setStandby={setRinnaStandby}
        />
      </div>
      <div>
        {conversationList.map((conversation) => (
          <ConversationTentativeElm
            key={conversation.id}
            id={conversation.id}
            prompt={conversation.prompt}
          />
        ))}
      </div>
    </ThemeProvider>
  );
}

export default App;
