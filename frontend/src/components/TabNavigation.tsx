import { Search, FileText } from 'lucide-react'

export type TabType = 'single' | 'batch'

interface TabNavigationProps {
  activeTab: TabType
  onTabChange: (tab: TabType) => void
}

export function TabNavigation({ activeTab, onTabChange }: TabNavigationProps) {
  const tabs = [
    {
      id: 'single' as TabType,
      name: '单个查询',
      icon: Search,
      description: '查询单个手机号码归属地'
    },
    {
      id: 'batch' as TabType,
      name: '批量查询',
      icon: FileText,
      description: '批量查询多个手机号码归属地'
    }
  ]

  return (
    <div className="bg-white rounded-xl shadow-sm border border-gray-100 p-1">
      <nav className="flex space-x-1">
        {tabs.map((tab) => {
          const Icon = tab.icon
          const isActive = activeTab === tab.id
          
          return (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`flex-1 flex items-center justify-center space-x-2 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200 ${
                isActive
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
              }`}
            >
              <Icon className="h-4 w-4" />
              <span className="hidden sm:inline">{tab.name}</span>
              <span className="sm:hidden">{tab.name.slice(0, 2)}</span>
            </button>
          )
        })}
      </nav>
      
      {/* 活动标签描述 */}
      <div className="mt-2 px-4 py-2">
        <p className="text-xs text-gray-500 text-center">
          {tabs.find(tab => tab.id === activeTab)?.description}
        </p>
      </div>
    </div>
  )
}