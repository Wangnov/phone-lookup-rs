import { useState } from 'react'
import { Header } from './components/Header'
import { Footer } from './components/Footer'
import { TabNavigation, type TabType } from './components/TabNavigation'
import { SingleQuery } from './components/SingleQuery'
import { BatchQuery } from './components/BatchQuery'

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('single')

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      <Header />
      
      <main className="flex-1">
        <div className="max-w-4xl mx-auto px-4 py-8 space-y-8">
          {/* 标签导航 */}
          <TabNavigation 
            activeTab={activeTab} 
            onTabChange={setActiveTab} 
          />
          
          {/* 主要内容区域 */}
          <div className="space-y-8">
            {activeTab === 'single' && <SingleQuery />}
            {activeTab === 'batch' && <BatchQuery />}
          </div>
        </div>
      </main>
      
      <Footer />
    </div>
  )
}

export default App
