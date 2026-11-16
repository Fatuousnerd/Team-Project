import { Outlet } from "react-router";
import { AppSidebar } from "@/components/app/app-sidebar";
import { TopBar } from "@/components/app/top-bar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useMarketStream } from "@/hooks/useMarketStream";

export function AppShell() {
	useMarketStream();

	return (
		<SidebarProvider defaultOpen>
			<AppSidebar />
			<SidebarInset className="min-w-0">
				<TopBar />
				<main className="flex-1">
					<Outlet />
				</main>
			</SidebarInset>
		</SidebarProvider>
	);
}

export default AppShell;
