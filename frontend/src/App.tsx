import { Header, Heading, Grid, View } from '@adobe/react-spectrum'
import { Sidebar } from './components/Sidebar'
import { ChatWindow } from './components/ChatWindow'


function App() {

  return (
    <Grid
      areas={[
      'header  header',
      'nav content'
      ]}
      columns={['1fr', '3fr']}
      rows={['size-1000', 'auto', 'size-1000']}
      width="100wh"
      height="100vh"
      gap="size-100"
    >
      <Header gridArea={'header'} margin='size-100'>
        <Heading level={1}>Web App Demo</Heading>
      </Header>
      <View gridArea={'nav'}>
        <Sidebar />
      </View>
      <ChatWindow gridArea={'content'} />
    </Grid>
  )
}

export default App
