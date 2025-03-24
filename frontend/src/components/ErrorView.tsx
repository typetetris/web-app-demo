import { Heading, Text, Content } from "@adobe/react-spectrum";
import { useRouteError } from "react-router";

export function ErrorView() {
  const error = useRouteError();
  console.error(error);
  return (
    <>
      <Heading level={1}>Error</Heading>
      <Content>
        <Text>{`${error}`}</Text>
      </Content>
    </>
  );
}
