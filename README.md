# WDEPC

Western Digital EPC(Extended Power Condition) control tools for Linux.

This tool is only tested on Western Digital `HC320` disk, but may work in other Western Digital disk supported `EPC`.

All query function is 99% safe, unless your disk translate those command into some wrong `WRITE` command.

**USE AT YOUR RISK.**


## What is EPC (Extended Power Condition)

### APM - Advanced Power Management
There was a power management mechanism called `APM` - `Advanced Power Management` in the late 90s.

It is supported by almost all hard drives.

APM defines a `APM Levels` from 0 - 255.

| level | description |
| --- | --- |
| 0 | Reserved |
| 1 | Minimum power consumption with Standby |
| 2 - 127 | Intermediate power management levels with Standby |
| 128 | Minimum power consumption without Standby |
| 129 - 254 | Intermediate power management levels without Standby |
| 254 | Maximum performance |
| 255 | Reserved |

where `standby` means spin down.


### EPC - Extended Power Condition
This is the latest power management standard in hard drives, it's usually supported on enterprise-grade hard drives (some newer hard drives don't support APM, EPC is used exclusively).

EPC defines two main state:

1. PM1: Idle state
   1. **Idle_a**: drive ready, not performing I/O; drive may power down some eletronics to reduce power without increasing response time.
   2. **Idle_b**: spindle rotation at 7200 with heads unloaded.
   3. **Idle_c**: spindle rotation at low RPM with heads unloaded.
2. PM2: Standby state
   1. **Standby_y**: same as Idle_c in Seagate and WD
   2. **Standby_Z**: Actuator is unloaded and spindle motor is stopped. Commands can be received immediately.

a SATA state `sleep`, same as Standby_z but require soft reset or hard reset to return to mode Standby_Z.

Current tools like `hdparm` can not update EPC settings like timer, enable or disable, so this tool came up.

## Usage

### Check Power Mode
get current power mode

```wdepc -d /dev/sda check```

Output:
```
idle a
````

### Enable EPC
Enable EPC and disable APM.

**The APM is disabled automatically and can not be controlled.**
```shell
wdepc -d /dev/sda enable
```

### Disable EPC
Disable EPC, but **doesn't re-enable APM**.

You must enable APM **MANUALLY** on demand.
```shell
wdepc -d /dev/sda disable
```

### Show EPC settings
Show EPC settings, include timer, state

```shell
wdepc -d /dev/sda info
```

Output:
```shell
* = enabled
All times are in 100 milliseconds

Name       Current Timer Default Timer Saved Timer Recovery Time Changeable Savable
Idle A     *20           *20           *20         1             true       true
Idle B     *6000         *6000         *6000       10            true       true
Idle C     0             0             0           40            true       true
Standby Y  0             0             0           40            true       true
Standby Z  0             0             0           150           true       true
```

### Force device goto a state
```shell
wdepc -d /dev/sda set <idle_a | idle_b | idle_c | standby_y | standby_z >
```

### Set timer
Set specific mode timer.

```shell
wdepc -d /dev/sda set-timer <mode> <timer> --save --enable true
```

If `--save` present, save the timer setting even after reboot.

`--enable` controls if the timer is enabled.

### Set state
Enable or disable a specific mode.

```shell
wdepc -d /dev/sda set-state <mode> --save --enable true
```

If `--save` present, save the state setting even after reboot.

`--enable` controls if the power state is enabled.

### Restore settings
Restore a specific power mode setting.

```shell
wdepc -d /dev/sda restore -d -s <mode>
```

If `--default` present, set current setting to default, else set current setting to saved setting.

If `--save` present, save current setting.

# Reference
1. [HC320 SATA spec](https://documents.westerndigital.com/content/dam/doc-library/en_us/assets/public/western-digital/product/data-center-drives/ultrastar-dc-hc300-series/product-manual-ultrastar-dc-hc320-sata-oem-spec.pdf)
2. https://serverfault.com/a/1047332
3. [Seagate SCSI Reference](https://www.seagate.com/files/staticfiles/support/docs/manual/Interface%20manuals/100293068k.pdf)