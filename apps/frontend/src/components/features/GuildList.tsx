import { ChevronRight, Shield } from "lucide-react";
import { Avatar } from "@/components/ui/avatar";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { useApp } from "@/contexts/AppContext";

export function GuildList() {
  const { guilds, selectedGuild, setSelectedGuild, setSidebarOpen, setView } = useApp();

  if (guilds.loading) {
    return (
      <div className="px-3 space-y-2">
        <Skeleton className="h-9 w-full" />
        <Skeleton className="h-9 w-full" />
        <Skeleton className="h-9 w-full" />
      </div>
    );
  }

  if (!guilds.loading && guilds.data?.length === 0) {
    return (
      <div className="px-3 py-4 text-center border-2 border-dashed rounded-lg">
        <Shield className="h-5 w-5 mx-auto text-muted-foreground mb-2" />
        <p className="text-xs text-muted-foreground">No admin guilds found</p>
      </div>
    );
  }

  return (
    <>
      {guilds.data?.map((guild) => (
        <button
          key={guild.id}
          onClick={() => {
            setSelectedGuild(guild.id);
            setView("guild");
            if (window.innerWidth < 1024) setSidebarOpen(false);
          }}
          className={cn(
            "group flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
            selectedGuild === guild.id
              ? "bg-sidebar-primary text-sidebar-primary-foreground shadow-sm"
              : "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground text-muted-foreground",
          )}
        >
          <Avatar
            name={guild.name}
            guildId={guild.id}
            iconHash={guild.icon}
            className={cn(
              "h-6 w-6 text-[10px]",
              selectedGuild === guild.id
                ? "bg-sidebar-primary-foreground/20 text-sidebar-primary-foreground"
                : "bg-muted",
            )}
          />
          <span className="truncate">{guild.name}</span>
          {selectedGuild === guild.id && <ChevronRight className="ml-auto h-4 w-4 opacity-50" />}
        </button>
      ))}
    </>
  );
}
