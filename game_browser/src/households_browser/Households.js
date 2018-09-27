import React from 'react';
import * as cityboundBrowser from '../../Cargo.toml';
import update from 'immutability-helper';
const EL = React.createElement;

export const initialState = {
    buildingPositions: {},
    inspectedBuilding: null,
    inspectedBuildingState: null,
    householdInfo: {},
};

export function render(state, setState) {
    if (state.uiMode != "inspection") {
        return {}
    }

    const { inspectedBuilding, inspectedBuildingState, householdInfo } = state.households;

    const windows = inspectedBuilding && EL(BuildingInfo, { inspectedBuilding, inspectedBuildingState, householdInfo });

    const interactables = Object.keys(state.households.buildingPositions).map(buildingId => {
        const buildingPosition2d = state.households.buildingPositions[buildingId];
        const buildingPosition = [buildingPosition2d[0], buildingPosition2d[1], 0];

        return {
            id: buildingId,
            shape: {
                type: "circle",
                center: buildingPosition,
                radius: 3
            },
            zIndex: 2,
            cursorHover: "pointer",
            cursorActive: "pointer",
            onEvent: e => {
                if (e.drag && e.drag.end) {
                    setState(oldState => update(oldState, {
                        households: { inspectedBuilding: { $set: buildingId } }
                    }))
                }
            }
        }
    })

    return { windows, interactables };
}

class BuildingInfo extends React.Component {
    constructor(props) {
        super(props);

        this.refresh = () => {
            cityboundBrowser.get_building_info(this.props.inspectedBuilding);
            if (this.props.inspectedBuildingState) {
                for (let householdId of this.props.inspectedBuildingState.households) {
                    cityboundBrowser.get_household_info(householdId);
                }
            }
        }
    }

    componentWillMount() {
        this.refresh();
        this.refreshInterval = setInterval(this.refresh, 1000);
    }

    componentWillUnmount() {
        clearInterval(this.refreshInterval);
    }

    render() {
        return EL("div", { className: "window building" }, [
            EL("h1", {}, this.props.inspectedBuilding),
            this.props.inspectedBuildingState && [
                EL("h2", {}, "Building" + this.props.inspectedBuildingState.style),
                this.props.inspectedBuildingState.households.map(id => [
                    EL("h4", {}, "Household " + id),
                    this.props.householdInfo[id] && EL("p", {}, this.props.householdInfo[id].core.member_resources.length + " members")
                ])
            ]
        ]);
    }
}

