import { Button, Flex, Form, TextField } from "@adobe/react-spectrum";
import Add from "@spectrum-icons/workflow/Add";
import Close from "@spectrum-icons/workflow/Close";
import { useState } from "react";

export interface CreateNewChatFormProps {
  onSubmit: (name: string) => void;
}
export function CreateNewChatForm({ onSubmit }: CreateNewChatFormProps) {
  const [name, setName] = useState("");
  const clearForm = () => {
    setName("");
  };
  return (
    <Form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit(name);
        clearForm();
      }}
      onReset={clearForm}
    >
      <Flex direction="row" alignItems="end" gap="size-100">
        <TextField
          name="name"
          label="Chat Name"
          value={name}
          onChange={setName}
        />
        <Button
          type="submit"
          variant="accent"
          isDisabled={name === ""}
          aria-label="Add display name"
        >
          <Add />
        </Button>
        <Button
          type="reset"
          variant="secondary"
          aria-label="Clear display name input field"
        >
          <Close />
        </Button>
      </Flex>
    </Form>
  );
}
