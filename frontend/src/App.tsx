import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { AppShell } from './components/app-shell'
import { IncidentsPage } from './pages/incidents'
import { IncidentDetailPage } from './pages/incident-detail'

const router = createBrowserRouter([
  {
    element: <AppShell />,
    children: [
      { path: '/', element: <IncidentsPage /> },
      { path: '/:incidentId', element: <IncidentDetailPage /> },
      { path: '*', element: <IncidentsPage /> },
    ],
  },
])

const App = () => <RouterProvider router={router} />

export default App
