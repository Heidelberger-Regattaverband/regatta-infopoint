import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import Button, { Button$PressEvent } from "sap/m/Button";
import MessageToast from "sap/m/MessageToast";

/**
* @namespace de.regatta_hd.infoportal.controller
*/
export default class RaceRegistrationsTable extends BaseController {
  static formatter: Formatter

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    super.setViewModel(new JSONModel(), "raceRegistrations");

    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow }, this);

    super.getEventBus()?.subscribe("race", "itemChanged", this.onItemChanged, this);

    window.addEventListener("keydown", (event: KeyboardEvent) => {
      switch (event.key) {
        case "F5":
          event.preventDefault();
          break;
        case "ArrowLeft":
          this.onPreviousPress();
          break;
        case "ArrowRight":
          this.onNextPress();
          break;
        case "ArrowUp":
          this.onFirstPress();
          break;
        case "ArrowDown":
          this.onLastPress();
          break;
      }
    });
  }

  async onBeforeShow(): Promise<void> {
    await this.loadRaceModel();
  }

  onNavBack(): void {
    super.displayTarget("races");
  }

  onFirstPress(): void {
    super.getEventBus()?.publish("race", "first", {});
  }

  onPreviousPress(): void {
    super.getEventBus()?.publish("race", "previous", {});
  }

  onNextPress(): void {
    super.getEventBus()?.publish("race", "next", {});
  }

  onLastPress(): void {
    super.getEventBus()?.publish("race", "last", {});
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    await this.loadRaceModel();
    MessageToast.show(this.i18n("msg.dataUpdated"));
    source.setEnabled(true);
  }

  private async loadRaceModel(): Promise<void> {
    const race = this.getComponentModel("race") as JSONModel;
    await this.updateJSONModel(super.getViewModel("raceRegistrations") as JSONModel, `/api/races/${race.getData().id}`, undefined);
  }

  private async onItemChanged(channelId: string, eventId: string, parametersMap: any) {
    await this.loadRaceModel();
  }

}
