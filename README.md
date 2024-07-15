<p align="center">
<img src="https://github.com/appsinacup/godot-rapier-physics/blob/main/logo.jpg?raw=true"/>
</p>
<p align="center">
        <img src="https://github.com/appsinacup/godot-rapier-physics/actions/workflows/runner.yml/badge.svg?branch=main"
            alt="Godot Rapier Build"></a>
        <img src="https://img.shields.io/badge/Godot-4.2-%23478cbf?logo=godot-engine&logoColor=white" />
</p>

<p align = "center">
    <strong>
        <a href="https://github.com/appsinacup/godot-rapier-physics/blob/main/docs/CONTRIBUTING.md">Contributing</a> | <a href="https://discord.gg/56dMud8HYn">Discord</a>
    </strong>
</p>


-----

<p align = "center">
<b>2D and 3D physics engine</b>
<i>for the Godot game engine.</i>
</p>

-----

Godot Rapier Physics is a 2D and 3D physics drop-in replacement for the [Godot game engine](https://github.com/godotengine/godot) that adds stability through [rapier](https://github.com/dimforge/rapier) and fluids through [salva](https://github.com/dimforge/salva).

# Installation

- Automatic (Recommended): Download the plugin from the official [Godot Asset Store](https://godotengine.org/asset-library/asset/2267) using the `AssetLib` tab in Godot:
    - [Rapier Physics 2D - Parallel SIMD](https://godotengine.org/asset-library/asset/2267)
    - [Rapier Physics 2D - Cross Platform Determinism](https://godotengine.org/asset-library/asset/2815)
    - [Rapier Physics 3D - Parallel SIMD](https://godotengine.org/asset-library/asset/3084)
    - [Rapier Physics 3D - Cross Platform Determinism](https://godotengine.org/asset-library/asset/3085)

- Manual: Download the [latest github release](https://github.com/appsinacup/godot-rapier-physics/releases/latest) and move only the `addons` folder into your project `addons` folder.

After installing, go to `Advanced Settings` -> `Physics` -> `2D` or `3D`. Change `Physics Engine` to `Rapier2D` or `Rapier3D`.

<p align="center">
<img src="docs/rapier-vid.gif"/>
</p>

# Implementation Progress

This plugin is still being developed. See the [Implementation Progress](https://godot.rapier.rs/docs/progress/) to get an idea of what status it is in and what features it has.