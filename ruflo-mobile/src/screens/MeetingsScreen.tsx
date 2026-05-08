import { View, Text, StyleSheet, FlatList, TouchableOpacity } from 'react-native'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { api } from '../api/client'

interface Meeting {
  id: string
  title: string
  scheduled_start_at: string
  status: string
}

export function MeetingsScreen({ navigation }: any) {
  const { data: meetings } = useQuery({
    queryKey: ['meetings'],
    queryFn: async () => {
      const response = await api.get('/api/v1/meetings')
      return response.data.data as Meeting[]
    },
  })

  const renderItem = ({ item }: { item: Meeting }) => (
    <TouchableOpacity
      style={styles.meetingCard}
      onPress={() => navigation.navigate('MeetingDetail', { id: item.id })}
    >
      <Text style={styles.meetingTitle}>{item.title}</Text>
      <Text style={styles.meetingTime}>
        {format(new Date(item.scheduled_start_at), 'MMM d, yyyy h:mm a')}
      </Text>
      <View style={[styles.statusBadge, 
        item.status === 'completed' ? styles.statusCompleted : styles.statusScheduled
      ]}>
        <Text style={styles.statusText}>{item.status}</Text>
      </View>
    </TouchableOpacity>
  )

  return (
    <View style={styles.container}>
      <FlatList
        data={meetings}
        renderItem={renderItem}
        keyExtractor={(item) => item.id}
        contentContainerStyle={styles.list}
      />
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F9FAFB',
  },
  list: {
    padding: 16,
  },
  meetingCard: {
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  meetingTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#111827',
    marginBottom: 8,
  },
  meetingTime: {
    fontSize: 14,
    color: '#6B7280',
    marginBottom: 8,
  },
  statusBadge: {
    alignSelf: 'flex-start',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusScheduled: {
    backgroundColor: '#DBEAFE',
  },
  statusCompleted: {
    backgroundColor: '#D1FAE5',
  },
  statusText: {
    fontSize: 12,
    fontWeight: '500',
    color: '#374151',
  },
})
