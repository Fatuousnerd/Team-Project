import { createHashRouter } from "react-router";
import AppShell from "@/layouts/AppShell";
import Dashboard from "@/pages/Dashboard";
import Markets from "@/pages/Markets";
import NotFound from "@/pages/NotFound";
import Orders from "@/pages/Orders";
import Portfolio from "@/pages/Portfolio";
import Settings from "@/pages/Settings";
import Strategies from "@/pages/Strategies";
import SymbolPage from "@/pages/Symbol";

export const router = createHashRouter([
	{
		path: "/",
		element: <AppShell />,
		children: [
			{ index: true, element: <Dashboard /> },
			{ path: "markets", element: <Markets /> },
			{ path: "markets/:symbol", element: <SymbolPage /> },
			{ path: "portfolio", element: <Portfolio /> },
			{ path: "orders", element: <Orders /> },
			{ path: "strategies", element: <Strategies /> },
			{ path: "settings", element: <Settings /> },
			{ path: "*", element: <NotFound /> },
		],
	},
]);
