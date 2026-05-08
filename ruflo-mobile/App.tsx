import { StatusBar } from 'expo-status-bar'
import { NavigationContainer } from '@react-navigation/native'
import { createNativeStackNavigator } from '@react-navigation/native-stack'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { MainTabs } from './src/navigation/MainTabs'
import { LoginScreen } from './src/screens/LoginScreen'
import { MeetingDetailScreen } from './src/screens/MeetingDetailScreen'
import { useAuthStore } from './src/store/authStore'

const Stack = createNativeStackNavigator()
const queryClient = new QueryClient()

export default function App() {
  const { isAuthenticated } = useAuthStore()

  return (
    <QueryClientProvider client={queryClient}>
      <NavigationContainer>
        <Stack.Navigator screenOptions={{ headerShown: false }}>
          {!isAuthenticated ? (
            <Stack.Screen name="Login" component={LoginScreen} />
          ) : (
            <>
              <Stack.Screen name="Main" component={MainTabs} />
              <Stack.Screen
                name="MeetingDetail"
                component={MeetingDetailScreen}
                options={{ headerShown: true, title: 'Meeting' }}
              />
            </>
          )}
        </Stack.Navigator>
        <StatusBar style="auto" />
      </NavigationContainer>
    </QueryClientProvider>
  )
}
