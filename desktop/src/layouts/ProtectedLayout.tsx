import { Navigate, Outlet, useLocation } from "react-router";
import Loading from "@/components/Loading";
import { useUser } from "@/hooks/useUser";

export const ProtectedLayout = () => {
	const { user, loading } = useUser();
	const location = useLocation();

	if (loading) return <Loading />;

	if (!user) return <Navigate to={"auth"} replace state={{ from: location }} />;

	return <Outlet />;
};

export default ProtectedLayout;
