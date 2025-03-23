import {
  Button,
  Content,
  ContextualHelp,
  Flex,
  Form,
  Heading,
  Keyboard,
  TextArea,
  Tooltip,
  TooltipTrigger,
} from "@adobe/react-spectrum";
import Close from "@spectrum-icons/workflow/Close";
import Send from "@spectrum-icons/workflow/Send";
import { useRef, useState } from "react";
import { DOMRefValue, KeyboardEvent, StyleProps } from "@react-types/shared";

export interface SendChatMessageFormProps extends StyleProps {
  onSubmit: (name: string) => void;
  onScrollToBottom: () => void;
}
export function SendChatMessageForm({
  onSubmit,
  ...styleProps
}: SendChatMessageFormProps) {
  const [name, setName] = useState("");
  const formRef = useRef<DOMRefValue<HTMLFormElement>>(null);
  const clearForm = () => {
    setName("");
  };
  return (
    <>
      <Form
        onSubmit={(e) => {
          e.preventDefault();
          onSubmit(name);
          clearForm();
        }}
        onReset={clearForm}
        ref={formRef}
        {...styleProps}
      >
        <Flex direction="row" alignItems="center" gap="size-100">
          <TextArea
            aria-label="Nachrichteneditor"
            value={name}
            onChange={setName}
            flex="1 1 auto"
            autoFocus
            inputMode="text"
            spellCheck="true"
            onKeyDown={(e: KeyboardEvent) => {
              if (e.target == e.currentTarget) {
                if (e.getModifierState("Control") && e.key == "Enter") {
                  formRef.current?.UNSAFE_getDOMNode()?.requestSubmit();
                } else if (e.key == "Escape") {
                  clearForm();
                }
              }
            }}
          />
          <TooltipTrigger>
            <Button
              type="submit"
              variant="accent"
              isDisabled={name === ""}
              aria-label="Nachricht senden"
            >
              <Send />
            </Button>
            <Tooltip>
              <Keyboard>Ctrl+Enter</Keyboard>
            </Tooltip>
          </TooltipTrigger>
          <TooltipTrigger>
            <Button
              type="reset"
              variant="secondary"
              aria-label="Nachrichteneditor leeren"
            >
              <Close />
            </Button>
            <Tooltip>
              <Keyboard>Escape</Keyboard>
            </Tooltip>
          </TooltipTrigger>
        </Flex>
        <Flex direction="row">
          <ContextualHelp variant="info">
            <Heading>Keyboard Shortcuts</Heading>
            <Content>
              <dl>
                <dt>
                  <Keyboard>Ctrl+Enter</Keyboard>
                </dt>
                <dd>Nachricht absenden</dd>
                <dt>
                  <Keyboard>Escape</Keyboard>
                </dt>
                <dd>Nachrichteneditor leeren</dd>
                <dt>
                  <Keyboard>Ctrl+ArrowUp</Keyboard>
                </dt>
                <dd>Zur ältesten Nachricht scrollen</dd>
                <dt>
                  <Keyboard>Ctrl+ArrowDown</Keyboard>
                </dt>
                <dd>Zur aktuellsten Nachricht scrollen</dd>
              </dl>
            </Content>
          </ContextualHelp>
        </Flex>
      </Form>
    </>
  );
}
