import { DeviceList } from "@/components/DeviceList";
import { NetworkStats } from "@/components/NetworkStats";
import { WanIpDisplay } from "@/components/WanIpDisplay";
import { fetchNetworkWanInfo, fetchDevices } from "@/features";
import { useAppSelector } from "@/hooks";
import { useAppDispatch } from "@/hooks/use-dispatch";
import { useEffect, useRef, useState } from "react";

const Dashboard = () => {
  const dispatch = useAppDispatch();
  const [wanHasFailed, setWanHasFailed] = useState<boolean>(false);
  const [deviceHasFailed, setDeviceHasFailed] = useState<boolean>(false);

  const { wan, status: wanStatus } = useAppSelector((state) => state.network);
  const { devices, status: devicesStatus } = useAppSelector(
    (state) => state.devices,
  );

  const refreshInterval = 5000; // TODO: Settings

  const wanStatusRef = useRef(wanStatus);
  useEffect(() => {
    wanStatusRef.current = wanStatus;
  }, [wanStatus]);

  const deviceStatusRef = useRef(devicesStatus);
  useEffect(() => {
    deviceStatusRef.current = devicesStatus;
  }, [devicesStatus]);

  useEffect(() => {
    dispatch(fetchNetworkWanInfo());
    dispatch(fetchDevices());

    const intervalId = setInterval(() => {
      if (wanStatusRef.current !== "loading") {
        setWanHasFailed(wanStatusRef.current === "failed");
        dispatch(fetchNetworkWanInfo());
      }
      if (deviceStatusRef.current !== "loading") {
        setDeviceHasFailed(deviceStatusRef.current === "failed");
        dispatch(fetchDevices());
      }
    }, refreshInterval);

    return () => clearInterval(intervalId);
  }, [dispatch, refreshInterval, setWanHasFailed, setDeviceHasFailed]);

  return (
    <>
      <NetworkStats hasFailed={wanHasFailed} wan={wan} />
      <WanIpDisplay hasFailed={wanHasFailed} wan={wan} />
      <DeviceList
        hasFailed={deviceHasFailed}
        devices={devices}
        status={devicesStatus}
      />
    </>
  );
};

export default Dashboard;
