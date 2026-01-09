import { LogOut, Settings, ChevronsUpDown } from "lucide-react";
import { Avatar } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";
import { useApp } from "@/contexts/AppContext";
import { redirectToLogin } from "@/lib/api";
import { GuildList } from "@/components/features/GuildList";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";

export function Sidebar() {
  const { session, sidebarOpen, setView } = useApp();

  if (!session) return null;

  return (
    <aside
      className={cn(
        "fixed inset-y-0 left-0 z-50 flex w-72 flex-col border-r bg-sidebar text-sidebar-foreground transition-transform duration-300 lg:relative lg:translate-x-0",
        sidebarOpen ? "translate-x-0" : "-translate-x-full",
      )}
    >
      <div
        className="flex h-16 items-center gap-2 border-b px-6 cursor-pointer"
        onClick={() => setView("guild")}
      >
        <img
          src="/logo.png"
          alt="Logo"
          className="h-8 w-8 rounded-lg object-cover"
        />
        <div className="font-semibold tracking-tight"> flora </div>
      </div>

      <div className="flex-1 overflow-y-auto py-4 px-3 space-y-1">
        <div className="px-3 mb-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          Your Guilds
        </div>
        <GuildList />
      </div>

      <div className="border-t p-4">
        <DropdownMenu>
          <DropdownMenuTrigger className="flex w-full items-center gap-3 rounded-lg border bg-sidebar-accent/50 p-3 shadow-sm hover:bg-sidebar-accent transition-colors outline-none text-left cursor-pointer">
            <Avatar
              name={session.global_name || session.username}
              userId={session.id}
              avatarHash={session.avatar}
              className="h-8 w-8"
            />
            <div className="flex-1 min-w-0">
              <p className="truncate text-sm font-medium">
                {" "}
                {session.global_name || session.username}{" "}
              </p>
              <p className="truncate text-xs text-muted-foreground">
                {" "}
                Manage Account{" "}
              </p>
            </div>
            <ChevronsUpDown className="h-4 w-4 text-muted-foreground opacity-50" />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" side="top" className="w-[220px]">
            <DropdownMenuItem onClick={() => setView("user-settings")}>
              <Settings className="mr-2 h-4 w-4" />
              <span>Settings </span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={redirectToLogin}
              className="text-destructive focus:text-destructive"
            >
              <LogOut className="mr-2 h-4 w-4" />
              <span>Sign out </span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </aside>
  );
}
