import { createBrowserRouter } from "react-router";
import ProtectedLayout from "@/layouts/ProtectedLayout";
import Home from "@/pages/Home";

export const router = createBrowserRouter([
	{
		path: "/",
		element: <ProtectedLayout />,
		children: [{ index: true, element: <Home /> }],
	},
]);
