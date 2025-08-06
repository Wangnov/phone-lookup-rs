import { Github, Heart } from 'lucide-react'

export function Footer() {
  return (
    <footer className="bg-white border-t border-gray-200 mt-auto">
      <div className="max-w-4xl mx-auto px-4 py-6">
        <div className="flex items-center justify-between text-sm text-gray-600">
          <div className="flex items-center space-x-4">
            <span>© 2025 手机号归属地查询工具</span>
            <span className="text-gray-400">|</span>
            <span>基于 Rust + Tauri + React 构建</span>
          </div>
          
          <div className="flex items-center space-x-4">
            <div className="flex items-center">
              <span>Made with</span>
              <Heart className="h-4 w-4 mx-1 text-red-500" fill="currentColor" />
              <span>in China</span>
            </div>
            <button
              className="flex items-center hover:text-gray-900 transition-colors"
              onClick={() => window.open('https://github.com', '_blank')}
            >
              <Github className="h-4 w-4 mr-1" />
              GitHub
            </button>
          </div>
        </div>
        
        <div className="mt-4 pt-4 border-t border-gray-100 text-xs text-gray-500">
          <div className="flex flex-wrap items-center gap-4">
            <span>• 本地数据库查询，无需网络连接</span>
            <span>• 支持单个和批量查询</span>
            <span>• 查询速度 &lt; 1ms</span>
            <span>• 支持CSV导出</span>
          </div>
        </div>
      </div>
    </footer>
  )
}