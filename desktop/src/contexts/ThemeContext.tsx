import Cookies from "js-cookie";
import { createContext, type ReactNode, useEffect, useState } from "react";
import type { ThemeType } from "@/config/Types/app.types";

export const ThemeContext = createContext<ThemeType | null>(null);

export const ThemeProvider = ({ children }: { children: ReactNode }) => {
	const [theme, setTheme] = useState<ThemeType["theme"]>("light");

	useEffect(() => {
		const prefersDark = window.matchMedia(
			"(prefers-color-scheme:dark)",
		).matches;
		const themeGet = Cookies.get("theme");

		if (!themeGet) {
			const initialTheme = prefersDark ? "dark" : "light";
			Cookies.set("theme", initialTheme, { expires: 365 });
			setTheme(initialTheme);
		} else setTheme(themeGet);
	}, []);

	useEffect(() => {
		document.body.className = theme;
	}, [theme]);

	const toggleTheme: ThemeType["toggleTheme"] = () => {
		const newTheme = theme === "dark" ? "light" : "dark";
		Cookies.set("theme", newTheme, { expires: 365 });
		setTheme(newTheme);
	};

	return (
		<ThemeContext.Provider value={{ theme, toggleTheme }}>
			{children}
		</ThemeContext.Provider>
	);
};
