import "./App.css";
import { RouterProvider } from "react-router";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useTheme } from "@/hooks/useTheme";
import { router } from "@/lib/router";

function App() {
	const { theme } = useTheme();

	return (
		<TooltipProvider delayDuration={250}>
			<RouterProvider router={router} />
			<Toaster
				theme={theme}
				position="bottom-right"
				closeButton
				toastOptions={{ classNames: { toast: "num" } }}
			/>
		</TooltipProvider>
	);
}

export default App;
